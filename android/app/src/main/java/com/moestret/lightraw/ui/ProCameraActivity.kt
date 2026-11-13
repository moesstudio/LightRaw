package com.moestret.lightraw.ui

import android.Manifest
import android.annotation.SuppressLint
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.graphics.ImageFormat
import android.graphics.SurfaceTexture
import android.hardware.camera2.CameraCaptureSession
import android.hardware.camera2.CameraCharacteristics
import android.hardware.camera2.CameraDevice
import android.hardware.camera2.CameraManager
import android.hardware.camera2.CaptureRequest
import android.hardware.camera2.DngCreator
import android.hardware.camera2.TotalCaptureResult
import android.media.ImageReader
import android.net.Uri
import android.os.Bundle
import android.os.Handler
import android.os.HandlerThread
import android.view.Gravity
import android.view.Surface
import android.view.TextureView
import android.view.View
import android.widget.LinearLayout
import android.widget.SeekBar
import android.widget.TextView
import android.widget.Toast
import androidx.activity.result.contract.ActivityResultContracts
import androidx.appcompat.app.AppCompatActivity
import androidx.core.content.ContextCompat
import com.moestret.lightraw.R
import com.moestret.lightraw.data.CatalogDbHelper
import com.moestret.lightraw.data.MediaStoreImporter
import com.moestret.lightraw.databinding.ActivityCameraBinding
import com.moestret.lightraw.model.DevelopParams
import com.moestret.lightraw.model.PhotoAsset
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.io.File
import java.io.FileOutputStream
import java.util.Locale

/**
 * S11 Pro RAW/JPEG Camera Capture via Android Camera2 API with EV compensation,
 * White Balance presets, Flash mode, Rule-of-Thirds Grid, and Last-Capture preview.
 * Auto-imports captured photos into LightRawCatalog.lrcat and opens Loupe Editor.
 */
class ProCameraActivity : AppCompatActivity() {

    private enum class FlashOption(val label: String, val flashMode: Int) {
        OFF("Flash Off", CaptureRequest.FLASH_MODE_OFF),
        SINGLE("Flash Shot", CaptureRequest.FLASH_MODE_SINGLE),
        TORCH("Torch On", CaptureRequest.FLASH_MODE_TORCH)
    }

    private enum class WbOption(val label: String, val awbMode: Int) {
        AUTO("AWB", CaptureRequest.CONTROL_AWB_MODE_AUTO),
        DAYLIGHT("Daylight", CaptureRequest.CONTROL_AWB_MODE_DAYLIGHT),
        CLOUDY("Cloudy", CaptureRequest.CONTROL_AWB_MODE_CLOUDY_DAYLIGHT),
        TUNGSTEN("Tungsten", CaptureRequest.CONTROL_AWB_MODE_INCANDESCENT),
        FLUORESCENT("Fluor", CaptureRequest.CONTROL_AWB_MODE_FLUORESCENT)
    }

    private lateinit var binding: ActivityCameraBinding
    private lateinit var catalog: CatalogDbHelper
    private lateinit var cameraManager: CameraManager
    private val uiScope = CoroutineScope(Dispatchers.Main + Job())

    private var cameraId = "0"
    private var lensFacing = CameraCharacteristics.LENS_FACING_BACK
    private var supportsRaw = false
    private var cameraDevice: CameraDevice? = null
    private var captureSession: CameraCaptureSession? = null
    private var previewBuilder: CaptureRequest.Builder? = null
    private var jpegReader: ImageReader? = null
    private var rawReader: ImageReader? = null
    private var lastCharacteristics: CameraCharacteristics? = null
    private var latestResult: TotalCaptureResult? = null
    private var evIndex = 0
    private var evStep = 0.333f
    private var flashOption = FlashOption.OFF
    private var wbOption = WbOption.AUTO
    private var isGridVisible = false
    private var lastCapturedPhotoId: Long = -1L

    private var bgThread: HandlerThread? = null
    private var bgHandler: Handler? = null

    private val camPermLauncher =
        registerForActivityResult(ActivityResultContracts.RequestPermission()) { granted ->
            if (granted) openCameraIfReady() else {
                Toast.makeText(this, "Camera permission required", Toast.LENGTH_SHORT).show()
                finish()
            }
        }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        binding = ActivityCameraBinding.inflate(layoutInflater)
        setContentView(binding.root)
        catalog = CatalogDbHelper(this)
        cameraManager = getSystemService(Context.CAMERA_SERVICE) as CameraManager

        binding.btnCamClose.setOnClickListener { finish() }
        binding.btnCamSwitch.setOnClickListener {
            lensFacing = if (lensFacing == CameraCharacteristics.LENS_FACING_BACK) {
                CameraCharacteristics.LENS_FACING_FRONT
            } else {
                CameraCharacteristics.LENS_FACING_BACK
            }
            closeCamera()
            openCameraIfReady()
        }

        binding.btnCamFlash.setOnClickListener {
            val entries = FlashOption.entries
            flashOption = entries[(flashOption.ordinal + 1) % entries.size]
            binding.btnCamFlash.text = flashOption.label
            binding.btnCamFlash.setTextColor(
                ContextCompat.getColor(
                    this,
                    if (flashOption == FlashOption.OFF) R.color.lr_text_secondary else R.color.lr_warning
                )
            )
            applyPreviewSettings()
        }

        binding.btnCamGrid.setOnClickListener {
            isGridVisible = !isGridVisible
            binding.cameraGridOverlay.visibility = if (isGridVisible) View.VISIBLE else View.GONE
            binding.btnCamGrid.setTextColor(
                ContextCompat.getColor(
                    this,
                    if (isGridVisible) R.color.lr_accent_light else R.color.lr_text_muted
                )
            )
        }

        binding.swCamRaw.setOnCheckedChangeListener { _, isChecked ->
            binding.tvCamFormatBadge.text = if (supportsRaw && isChecked) "PRO · DNG + JPEG" else "PRO · JPEG"
        }

        binding.btnShutter.setOnClickListener { takePicture() }

        binding.ivLastCapture.setOnClickListener {
            if (lastCapturedPhotoId > 0L) {
                startActivity(
                    Intent(this, LoupeEditorActivity::class.java).apply {
                        putExtra("photo_id", lastCapturedPhotoId)
                    }
                )
            }
        }

        rebuildWbStrip()
        loadLastCaptureThumb()

        binding.sbCamEv.setOnSeekBarChangeListener(object : SeekBar.OnSeekBarChangeListener {
            override fun onProgressChanged(seekBar: SeekBar?, progress: Int, fromUser: Boolean) {
                if (!fromUser) return
                val range = lastCharacteristics?.get(CameraCharacteristics.CONTROL_AE_COMPENSATION_RANGE) ?: return
                val frac = progress / 24f
                evIndex = (range.lower + (range.upper - range.lower) * frac).toInt()
                binding.tvEvValue.text = String.format(Locale.US, "EV %+.1f", evIndex * evStep)
                applyPreviewSettings()
            }
            override fun onStartTrackingTouch(seekBar: SeekBar?) = Unit
            override fun onStopTrackingTouch(seekBar: SeekBar?) = Unit
        })

        binding.camPreview.surfaceTextureListener = object : TextureView.SurfaceTextureListener {
            override fun onSurfaceTextureAvailable(st: SurfaceTexture, w: Int, h: Int) {
                openCameraIfReady()
            }
            override fun onSurfaceTextureSizeChanged(st: SurfaceTexture, w: Int, h: Int) = Unit
            override fun onSurfaceTextureDestroyed(st: SurfaceTexture): Boolean = true
            override fun onSurfaceTextureUpdated(st: SurfaceTexture) = Unit
        }
    }

    private fun rebuildWbStrip() {
        binding.wbStripContainer.removeAllViews()
        for (opt in WbOption.entries) {
            val selected = opt == wbOption
            val pill = TextView(this).apply {
                text = opt.label
                textSize = 11.5f
                gravity = Gravity.CENTER
                setTextColor(ContextCompat.getColor(this@ProCameraActivity, if (selected) R.color.lr_on_accent else R.color.lr_text))
                setBackgroundResource(if (selected) R.drawable.bg_pill_accent else R.drawable.bg_pill_neutral)
                setPadding(
                    DarkroomUiFactory.dp(this@ProCameraActivity, 10),
                    0,
                    DarkroomUiFactory.dp(this@ProCameraActivity, 10),
                    0
                )
                layoutParams = LinearLayout.LayoutParams(
                    LinearLayout.LayoutParams.WRAP_CONTENT,
                    DarkroomUiFactory.dp(this@ProCameraActivity, 30)
                ).apply { marginEnd = DarkroomUiFactory.dp(this@ProCameraActivity, 6) }
                setOnClickListener {
                    wbOption = opt
                    binding.tvCamWbBadge.text = opt.label.uppercase(Locale.US)
                    rebuildWbStrip()
                    applyPreviewSettings()
                }
            }
            binding.wbStripContainer.addView(pill)
        }
    }

    private fun loadLastCaptureThumb() {
        uiScope.launch {
            val latest = withContext(Dispatchers.IO) { catalog.getAllPhotos().firstOrNull() } ?: return@launch
            lastCapturedPhotoId = latest.idLocal
            val bmp = withContext(Dispatchers.IO) {
                MediaStoreImporter.decodeScaledBitmap(this@ProCameraActivity, latest.uriString, 160)
            }
            if (bmp != null) {
                binding.ivLastCapture.setImageBitmap(bmp)
            }
        }
    }

    private fun applyPreviewSettings() {
        val b = previewBuilder ?: return
        b.set(CaptureRequest.CONTROL_AE_EXPOSURE_COMPENSATION, evIndex)
        b.set(CaptureRequest.CONTROL_AWB_MODE, wbOption.awbMode)
        b.set(
            CaptureRequest.FLASH_MODE,
            if (flashOption == FlashOption.TORCH) CaptureRequest.FLASH_MODE_TORCH else CaptureRequest.FLASH_MODE_OFF
        )
        runCatching { captureSession?.setRepeatingRequest(b.build(), null, bgHandler) }
    }

    override fun onResume() {
        super.onResume()
        bgThread = HandlerThread("LightRawCam").apply { start() }
        bgHandler = Handler(bgThread!!.looper)
        if (binding.camPreview.isAvailable) openCameraIfReady()
    }

    override fun onPause() {
        closeCamera()
        bgThread?.quitSafely()
        bgThread = null
        bgHandler = null
        super.onPause()
    }

    @SuppressLint("MissingPermission")
    private fun openCameraIfReady() {
        if (ContextCompat.checkSelfPermission(this, Manifest.permission.CAMERA) != PackageManager.PERMISSION_GRANTED) {
            camPermLauncher.launch(Manifest.permission.CAMERA)
            return
        }
        if (!binding.camPreview.isAvailable) return

        cameraId = findCameraId(lensFacing) ?: return
        val chars = cameraManager.getCameraCharacteristics(cameraId)
        lastCharacteristics = chars
        val caps = chars.get(CameraCharacteristics.REQUEST_AVAILABLE_CAPABILITIES) ?: IntArray(0)
        supportsRaw = CameraCharacteristics.REQUEST_AVAILABLE_CAPABILITIES_RAW in caps
        val step = chars.get(CameraCharacteristics.CONTROL_AE_COMPENSATION_STEP)
        if (step != null) evStep = step.toFloat()

        runOnUiThread {
            binding.swCamRaw.isEnabled = supportsRaw
            binding.swCamRaw.isChecked = supportsRaw
            binding.tvCamFormatBadge.text = if (supportsRaw) "PRO · DNG + JPEG" else "PRO · JPEG"
        }

        val map = chars.get(CameraCharacteristics.SCALER_STREAM_CONFIGURATION_MAP) ?: return
        val jpegSize = map.getOutputSizes(ImageFormat.JPEG).maxByOrNull { it.width * it.height } ?: return
        jpegReader = ImageReader.newInstance(jpegSize.width, jpegSize.height, ImageFormat.JPEG, 2).apply {
            setOnImageAvailableListener({ reader ->
                val img = reader.acquireLatestImage() ?: return@setOnImageAvailableListener
                val w = img.width
                val h = img.height
                val buf = img.planes[0].buffer
                val bytes = ByteArray(buf.remaining())
                buf.get(bytes)
                img.close()
                saveCapturedFile(bytes, isRaw = false, ext = "jpg", w = w, h = h)
            }, bgHandler)
        }

        if (supportsRaw) {
            val rawSize = map.getOutputSizes(ImageFormat.RAW_SENSOR).maxByOrNull { it.width * it.height }
            if (rawSize != null) {
                rawReader = ImageReader.newInstance(rawSize.width, rawSize.height, ImageFormat.RAW_SENSOR, 2).apply {
                    setOnImageAvailableListener({ reader ->
                        val img = reader.acquireLatestImage() ?: return@setOnImageAvailableListener
                        val res = latestResult
                        val c = lastCharacteristics
                        if (res != null && c != null && binding.swCamRaw.isChecked) {
                            val dir = File(filesDir, "captured").apply { mkdirs() }
                            val file = File(dir, "LR_CAM_${System.currentTimeMillis()}.dng")
                            runCatching {
                                FileOutputStream(file).use { out ->
                                    DngCreator(c, res).use { dng -> dng.writeImage(out, img) }
                                }
                            }
                        }
                        img.close()
                    }, bgHandler)
                }
            }
        }

        cameraManager.openCamera(cameraId, object : CameraDevice.StateCallback() {
            override fun onOpened(cam: CameraDevice) {
                cameraDevice = cam
                startPreviewSession(cam)
            }
            override fun onDisconnected(cam: CameraDevice) { cam.close(); cameraDevice = null }
            override fun onError(cam: CameraDevice, error: Int) { cam.close(); cameraDevice = null }
        }, bgHandler)
    }

    private fun findCameraId(targetFacing: Int): String? {
        for (id in cameraManager.cameraIdList) {
            val c = cameraManager.getCameraCharacteristics(id)
            if (c.get(CameraCharacteristics.LENS_FACING) == targetFacing) return id
        }
        return cameraManager.cameraIdList.firstOrNull()
    }

    @Suppress("DEPRECATION")
    private fun startPreviewSession(cam: CameraDevice) {
        val st = binding.camPreview.surfaceTexture ?: return
        st.setDefaultBufferSize(1280, 960)
        val previewSurface = Surface(st)
        val outputs = mutableListOf(previewSurface)
        jpegReader?.surface?.let { outputs.add(it) }
        rawReader?.surface?.let { if (supportsRaw) outputs.add(it) }

        val builder = cam.createCaptureRequest(CameraDevice.TEMPLATE_PREVIEW).apply {
            addTarget(previewSurface)
            set(CaptureRequest.CONTROL_MODE, CaptureRequest.CONTROL_MODE_AUTO)
            set(CaptureRequest.CONTROL_AE_EXPOSURE_COMPENSATION, evIndex)
            set(CaptureRequest.CONTROL_AWB_MODE, wbOption.awbMode)
            if (flashOption == FlashOption.TORCH) {
                set(CaptureRequest.FLASH_MODE, CaptureRequest.FLASH_MODE_TORCH)
            }
        }
        previewBuilder = builder

        cam.createCaptureSession(outputs, object : CameraCaptureSession.StateCallback() {
            override fun onConfigured(session: CameraCaptureSession) {
                captureSession = session
                runCatching { session.setRepeatingRequest(builder.build(), null, bgHandler) }
            }
            override fun onConfigureFailed(session: CameraCaptureSession) = Unit
        }, bgHandler)
    }

    private fun takePicture() {
        val cam = cameraDevice ?: return
        val session = captureSession ?: return
        val capture = cam.createCaptureRequest(CameraDevice.TEMPLATE_STILL_CAPTURE).apply {
            jpegReader?.surface?.let { addTarget(it) }
            if (supportsRaw && binding.swCamRaw.isChecked) {
                rawReader?.surface?.let { addTarget(it) }
            }
            set(CaptureRequest.CONTROL_AE_EXPOSURE_COMPENSATION, evIndex)
            set(CaptureRequest.CONTROL_AWB_MODE, wbOption.awbMode)
            set(CaptureRequest.FLASH_MODE, flashOption.flashMode)
            set(CaptureRequest.JPEG_QUALITY, 96.toByte())
        }
        session.capture(capture.build(), object : CameraCaptureSession.CaptureCallback() {
            override fun onCaptureCompleted(
                session: CameraCaptureSession,
                request: CaptureRequest,
                result: TotalCaptureResult
            ) {
                latestResult = result
            }
        }, bgHandler)
    }

    private fun saveCapturedFile(bytes: ByteArray, isRaw: Boolean, ext: String, w: Int, h: Int) {
        val dir = File(filesDir, "captured").apply { mkdirs() }
        val file = File(dir, "LR_CAM_${System.currentTimeMillis()}.$ext")
        FileOutputStream(file).use { it.write(bytes) }
        val saved = catalog.upsertPhoto(
            PhotoAsset(
                uriString = Uri.fromFile(file).toString(),
                displayName = file.name,
                width = w,
                height = h,
                isRaw = isRaw,
                albumName = "LightRaw Camera",
                params = DevelopParams()
            )
        )
        runOnUiThread {
            Toast.makeText(this, "Captured ${file.name}", Toast.LENGTH_SHORT).show()
            startActivity(
                Intent(this, LoupeEditorActivity::class.java).apply {
                    putExtra("photo_id", saved.idLocal)
                }
            )
            finish()
        }
    }

    private fun closeCamera() {
        runCatching { captureSession?.close() }
        captureSession = null
        runCatching { cameraDevice?.close() }
        cameraDevice = null
        jpegReader?.close(); jpegReader = null
        rawReader?.close(); rawReader = null
    }
}
