#import <Cocoa/Cocoa.h>
#import <CoreImage/CoreImage.h>
#import <CoreGraphics/CoreGraphics.h>
#import <ImageIO/ImageIO.h>
#import <Vision/Vision.h>
#import <CoreVideo/CoreVideo.h>
#import <Metal/Metal.h>
#include <stdatomic.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>

enum LightRawMenuAction {
    // 0. App Menu
    ACTION_ABOUT = 1,
    ACTION_SYSTEM_INFO = 2,
    ACTION_PREFERENCES = 3,

    // 1. File Menu
    ACTION_NEW_CATALOG = 10,
    ACTION_OPEN_CATALOG = 11,
    ACTION_OPTIMIZE_CATALOG = 12,
    ACTION_RENAME_CATALOG = 13,
    ACTION_IMPORT_PHOTOS = 14,
    ACTION_EXPORT_PHOTOS = 15,
    ACTION_EXPORT_PREVIOUS = 16,
    ACTION_SHOW_QUICK_COLLECTION = 17,
    ACTION_CLEAR_QUICK_COLLECTION = 18,
    ACTION_PAGE_SETUP = 19,
    ACTION_PRINTER = 20,
    ACTION_SAVE_QUICK_COLLECTION = 21,

    // 2. Edit Menu
    ACTION_UNDO = 30,
    ACTION_REDO = 31,
    ACTION_CUT = 32,
    ACTION_COPY = 33,
    ACTION_PASTE = 34,
    ACTION_DELETE = 35,
    ACTION_SELECT_ALL = 36,
    ACTION_SELECT_NONE = 37,
    ACTION_INVERT_SELECTION = 38,
    ACTION_SELECT_FLAGGED = 39,
    ACTION_DESELECT_UNFLAGGED = 40,
    ACTION_SELECT_ONLY_ACTIVE = 41,
    ACTION_DESELECT_ACTIVE = 42,
    ACTION_EDIT_WATERMARKS = 43,

    // 3. Develop / Library Menu
    ACTION_NEW_SNAPSHOT = 50,
    ACTION_NEW_PRESET = 51,
    ACTION_CLEAR_HISTORY = 52,
    ACTION_PREV_PHOTO = 53,
    ACTION_NEXT_PHOTO = 54,

    // 4. Photo Menu
    ACTION_QUICK_COLLECTION_NEXT = 60,
    ACTION_SHOW_IN_FINDER = 61,
    ACTION_CREATE_VIRTUAL_COPY = 62,
    ACTION_ROTATE_LEFT = 63,
    ACTION_ROTATE_RIGHT = 64,
    ACTION_FLIP_HORIZONTAL = 65,
    ACTION_FLIP_VERTICAL = 66,
    ACTION_SET_FLAG_PICK = 67,
    ACTION_SET_FLAG_UNFLAG = 68,
    ACTION_SET_FLAG_REJECT = 69,
    ACTION_SET_RATING_0 = 70,
    ACTION_SET_RATING_1 = 71,
    ACTION_SET_RATING_2 = 72,
    ACTION_SET_RATING_3 = 73,
    ACTION_SET_RATING_4 = 74,
    ACTION_SET_RATING_5 = 75,
    ACTION_SET_COLOR_NONE = 76,
    ACTION_SET_COLOR_RED = 77,
    ACTION_SET_COLOR_YELLOW = 78,
    ACTION_SET_COLOR_GREEN = 79,
    ACTION_SET_COLOR_BLUE = 80,
    ACTION_SET_COLOR_PURPLE = 81,
    ACTION_REMOVE_PHOTO = 82,
    ACTION_SAVE_METADATA = 83,
    ACTION_DELETE_REJECTED = 84,
    ACTION_ENHANCE_PHOTO = 85,

    // 5. Settings Menu
    ACTION_RESET_ALL_SETTINGS = 90,
    ACTION_COPY_SETTINGS = 91,
    ACTION_PASTE_SETTINGS = 92,
    ACTION_SYNC_SETTINGS = 93,
    ACTION_AUTO_SETTINGS = 94,
    ACTION_AUTO_WHITE_BALANCE = 95,
    ACTION_CONVERT_BW = 96,
    ACTION_TOGGLE_HDR = 97,
    ACTION_RESET_CROP = 98,
    ACTION_CONSTRAIN_ASPECT = 99,
    ACTION_SWAP_BEFORE_AFTER = 100,

    // 6. Tools Menu
    ACTION_TOOL_CROP = 110,
    ACTION_TOOL_RED_EYE = 111,
    ACTION_TOOL_REMOVE = 112,
    ACTION_TOOL_MASKING = 113,
    ACTION_TOOL_WB_SELECTOR = 114,
    ACTION_TOOL_TARGET_ADJUST = 115,
    ACTION_TOOL_GUIDED_UPRIGHT = 116,

    // 7. View Menu
    ACTION_HIDE_TOOLBAR = 120,
    ACTION_CYCLE_LOUPE = 121,
    ACTION_TOGGLE_ZOOM = 122,
    ACTION_ZOOM_IN = 123,
    ACTION_ZOOM_OUT = 124,
    ACTION_ZOOM_100 = 125,
    ACTION_BEFORE_AFTER = 126,
    ACTION_SOFT_PROOFING = 127,
    ACTION_SHOW_CLIPPING = 128,
    ACTION_VIEW_GRID = 129,
    ACTION_VIEW_LOUPE = 130,
    ACTION_VIEW_COMPARE = 131,
    ACTION_VIEW_SURVEY = 132,
    ACTION_VIEW_PEOPLE = 133,

    // 8. Window Menu
    ACTION_IMPROMPTU_SLIDESHOW = 140,
    ACTION_LIGHTS_OUT = 141,
    ACTION_TOGGLE_LEFT_PANEL = 142,
    ACTION_TOGGLE_RIGHT_PANEL = 143,
    ACTION_TOGGLE_FILMSTRIP = 144,
    ACTION_MODULE_LIBRARY = 150,
    ACTION_MODULE_DEVELOP = 151,
    ACTION_MODULE_MAP = 152,
    ACTION_MODULE_BOOK = 153,
    ACTION_MODULE_SLIDESHOW = 154,
    ACTION_MODULE_PRINT = 155,
    ACTION_MODULE_WEB = 156,

    // 9. Help Menu
    ACTION_WHATS_NEW = 160,
    ACTION_HELP_DOCUMENTATION = 161,
    ACTION_SHORTCUTS = 162,
    ACTION_CHECK_UPDATES = 163,

    // 10. Newly Wired Complete Menu Actions (Zero Dead Items)
    ACTION_OPEN_RECENT = 170,
    ACTION_IMPORT_CATALOG = 171,
    ACTION_IMPORT_PRESETS = 172,
    ACTION_EXPORT_WITH_PRESET = 173,
    ACTION_EXPORT_AS_CATALOG = 174,
    ACTION_EMAIL_PHOTOS = 175,
    ACTION_PLUGIN_MANAGER = 176,
    ACTION_SET_QUICK_COLLECTION_TARGET = 177,
    ACTION_GO_TO_FOLDER = 178,
    ACTION_EDIT_IN_EXTERNAL = 179,
    ACTION_PHOTO_MERGE_HDR = 180,
    ACTION_PHOTO_MERGE_PANO = 181,
    ACTION_READ_METADATA = 182,
    ACTION_UPDATE_DNG_PREVIEW = 183,
    ACTION_REMOVE_SELECTED_PHOTOS = 184,
    ACTION_PASTE_FROM_PREVIOUS = 185,
    ACTION_UPDATE_PROCESS_VERSION = 186,
    ACTION_TOOL_POINT_COLOR = 187,
    ACTION_OPEN_REFERENCE_VIEW = 188,
    ACTION_FILTER_FLAGGED = 189,
    ACTION_FILTER_RATED = 190,
    ACTION_FILTER_OFF = 191,
    ACTION_TETHERED_CAPTURE_INFO = 192,
    ACTION_AUTO_IMPORT_FOLDER = 193,
    ACTION_MANAGE_PRESETS = 194,
    ACTION_SYNC_CROP_TO_LIBRARY = 195
};

extern void lightraw_dispatch_menu_action(uint32_t action_id);

static atomic_int g_text_input_focused = 0;
static atomic_int g_menu_initialized = 0;

void lightraw_set_text_input_focused(int focused) {
    atomic_store(&g_text_input_focused, focused ? 1 : 0);
}

@interface LightRawMenuTarget : NSObject <NSMenuItemValidation>
+ (instancetype)shared;
- (void)onMenuAction:(NSMenuItem *)sender;
@end

@implementation LightRawMenuTarget
+ (instancetype)shared {
    static LightRawMenuTarget *instance = nil;
    static dispatch_once_t onceToken;
    dispatch_once(&onceToken, ^{
        instance = [[LightRawMenuTarget alloc] init];
    });
    return instance;
}

- (void)onMenuAction:(NSMenuItem *)sender {
    if (sender.tag > 0) {
        lightraw_dispatch_menu_action((uint32_t)sender.tag);
    }
}

- (BOOL)validateMenuItem:(NSMenuItem *)menuItem {
    if (menuItem.tag <= 0) {
        return NO;
    }
    // When an egui TextEdit has keyboard focus, yield single-key shortcuts and Cmd+A/C/V/Delete
    // to egui so typing, copying, pasting, and selecting text inside inputs works natively.
    if (atomic_load(&g_text_input_focused) != 0) {
        NSString *ke = menuItem.keyEquivalent;
        if (ke != nil && ke.length > 0) {
            NSEventModifierFlags mods = menuItem.keyEquivalentModifierMask;
            if ((mods & NSEventModifierFlagCommand) == 0) {
                return NO;
            }
            NSInteger tag = menuItem.tag;
            if (tag == ACTION_COPY || tag == ACTION_PASTE || tag == ACTION_SELECT_ALL || tag == ACTION_DELETE) {
                return NO;
            }
        }
    }
    return YES;
}
@end

static NSMenuItem *add_action_item(NSMenu *menu, NSString *title, NSString *key, NSEventModifierFlags flags, NSInteger tag) {
    NSMenuItem *item = [menu addItemWithTitle:title action:@selector(onMenuAction:) keyEquivalent:key];
    [item setTarget:[LightRawMenuTarget shared]];
    [item setTag:tag];
    // Always set modifier mask explicitly (even when flags == 0!) so single-key shortcuts
    // do NOT default to NSEventModifierFlagCommand and hijack Cmd+Q/W/M/C/V/S/O/N!
    [item setKeyEquivalentModifierMask:flags];
    return item;
}

static NSMenuItem *add_std_item(NSMenu *menu, NSString *title, SEL action, NSString *key, NSEventModifierFlags flags) {
    NSMenuItem *item = [menu addItemWithTitle:title action:action keyEquivalent:key];
    [item setKeyEquivalentModifierMask:flags];
    return item;
}

void setup_macos_native_menu(void) {
    dispatch_async(dispatch_get_main_queue(), ^{
        @autoreleasepool {
            NSApplication *app = [NSApplication sharedApplication];

            // Set Application Dock Icon
            NSString *iconPath = [[NSBundle mainBundle] pathForResource:@"LightRaw" ofType:@"icns"];
            if (!iconPath) {
                iconPath = [[NSBundle mainBundle] pathForResource:@"LightRaw" ofType:@"png"];
            }
            if (!iconPath && [[NSFileManager defaultManager] fileExistsAtPath:@"LightRaw.icns"]) {
                iconPath = @"LightRaw.icns";
            }
            if (!iconPath && [[NSFileManager defaultManager] fileExistsAtPath:@"assets/icon.png"]) {
                iconPath = @"assets/icon.png";
            }
            if (iconPath) {
                NSImage *appIcon = [[NSImage alloc] initWithContentsOfFile:iconPath];
                if (appIcon) {
                    [app setApplicationIconImage:appIcon];
                }
            }

            NSMenu *mainMenu = [[NSMenu alloc] initWithTitle:@"MainMenu"];

            // ====================================================================
            // 0. App Menu ( LightRaw)
            // ====================================================================
        NSMenuItem *appTopItem = [[NSMenuItem alloc] init];
        [mainMenu addItem:appTopItem];
        NSMenu *appMenu = [[NSMenu alloc] initWithTitle:@"LightRaw"];
        add_action_item(appMenu, @"About LightRaw", @"", 0, ACTION_ABOUT);
        add_action_item(appMenu, @"System Info...", @"", 0, ACTION_SYSTEM_INFO);
        [appMenu addItem:[NSMenuItem separatorItem]];
        add_action_item(appMenu, @"Settings...", @",", NSEventModifierFlagCommand, ACTION_PREFERENCES);
        [appMenu addItem:[NSMenuItem separatorItem]];
        NSMenuItem *servicesItem = [appMenu addItemWithTitle:@"Services" action:nil keyEquivalent:@""];
        NSMenu *servicesMenu = [[NSMenu alloc] initWithTitle:@"Services"];
        [servicesItem setSubmenu:servicesMenu];
        [app setServicesMenu:servicesMenu];
        [appMenu addItem:[NSMenuItem separatorItem]];
        add_std_item(appMenu, @"Hide LightRaw", @selector(hide:), @"h", NSEventModifierFlagCommand);
        add_std_item(appMenu, @"Hide Others", @selector(hideOtherApplications:), @"h", NSEventModifierFlagCommand | NSEventModifierFlagOption);
        add_std_item(appMenu, @"Show All", @selector(unhideAllApplications:), @"", 0);
        [appMenu addItem:[NSMenuItem separatorItem]];
        add_std_item(appMenu, @"Quit LightRaw", @selector(terminate:), @"q", NSEventModifierFlagCommand);
        [appTopItem setSubmenu:appMenu];

        // ====================================================================
        // 1. File Menu (Screenshot 18)
        // ====================================================================
        NSMenuItem *fileTopItem = [[NSMenuItem alloc] init];
        [mainMenu addItem:fileTopItem];
        NSMenu *fileMenu = [[NSMenu alloc] initWithTitle:@"File"];
        add_action_item(fileMenu, @"New Catalog...", @"", 0, ACTION_NEW_CATALOG);
        add_action_item(fileMenu, @"Open Catalog...", @"o", NSEventModifierFlagCommand | NSEventModifierFlagShift, ACTION_OPEN_CATALOG);
        add_action_item(fileMenu, @"Open Recent Catalog", @"", 0, ACTION_OPEN_RECENT);
        add_action_item(fileMenu, @"Optimize Catalog...", @"", 0, ACTION_OPTIMIZE_CATALOG);
        add_action_item(fileMenu, @"Rename Catalog...", @"", 0, ACTION_RENAME_CATALOG);
        [fileMenu addItem:[NSMenuItem separatorItem]];
        add_action_item(fileMenu, @"Import Photos and Video...", @"i", NSEventModifierFlagCommand | NSEventModifierFlagShift, ACTION_IMPORT_PHOTOS);
        add_action_item(fileMenu, @"Import from Another Catalog...", @"", 0, ACTION_IMPORT_CATALOG);
        add_action_item(fileMenu, @"Auto Import from Folder...", @"", 0, ACTION_AUTO_IMPORT_FOLDER);
        add_action_item(fileMenu, @"Tethered Capture...", @"", 0, ACTION_TETHERED_CAPTURE_INFO);
        add_action_item(fileMenu, @"Import Develop Profiles and Presets...", @"", 0, ACTION_IMPORT_PRESETS);
        add_action_item(fileMenu, @"Manage Presets...", @"", 0, ACTION_MANAGE_PRESETS);
        [fileMenu addItem:[NSMenuItem separatorItem]];
        add_action_item(fileMenu, @"Export...", @"e", NSEventModifierFlagCommand | NSEventModifierFlagShift, ACTION_EXPORT_PHOTOS);
        add_action_item(fileMenu, @"Export with Previous", @"e", NSEventModifierFlagCommand | NSEventModifierFlagOption | NSEventModifierFlagShift, ACTION_EXPORT_PREVIOUS);
        add_action_item(fileMenu, @"Export Full-Size JPEG...", @"", 0, ACTION_EXPORT_WITH_PRESET);
        add_action_item(fileMenu, @"Export as Catalog...", @"", 0, ACTION_EXPORT_AS_CATALOG);
        add_action_item(fileMenu, @"Email Photos...", @"m", NSEventModifierFlagCommand | NSEventModifierFlagShift, ACTION_EMAIL_PHOTOS);
        [fileMenu addItem:[NSMenuItem separatorItem]];
        add_action_item(fileMenu, @"Plug-in Manager...", @",", NSEventModifierFlagCommand | NSEventModifierFlagOption | NSEventModifierFlagShift, ACTION_PLUGIN_MANAGER);
        [fileMenu addItem:[NSMenuItem separatorItem]];
        add_action_item(fileMenu, @"Show Quick Collection", @"b", NSEventModifierFlagCommand, ACTION_SHOW_QUICK_COLLECTION);
        add_action_item(fileMenu, @"Save Quick Collection...", @"b", NSEventModifierFlagCommand | NSEventModifierFlagOption, ACTION_SAVE_QUICK_COLLECTION);
        add_action_item(fileMenu, @"Clear Quick Collection", @"b", NSEventModifierFlagCommand | NSEventModifierFlagShift, ACTION_CLEAR_QUICK_COLLECTION);
        add_action_item(fileMenu, @"Set Quick Collection as Target", @"b", NSEventModifierFlagCommand | NSEventModifierFlagOption | NSEventModifierFlagShift, ACTION_SET_QUICK_COLLECTION_TARGET);
        [fileMenu addItem:[NSMenuItem separatorItem]];
        NSMenuItem *libFiltItem = [fileMenu addItemWithTitle:@"Library Filters" action:nil keyEquivalent:@""];
        NSMenu *libFiltMenu = [[NSMenu alloc] initWithTitle:@"Library Filters"];
        add_action_item(libFiltMenu, @"Filter by Flagged", @"", 0, ACTION_FILTER_FLAGGED);
        add_action_item(libFiltMenu, @"Filter by Rated (>= 1★)", @"", 0, ACTION_FILTER_RATED);
        add_action_item(libFiltMenu, @"Filters Off", @"\\", NSEventModifierFlagCommand, ACTION_FILTER_OFF);
        [libFiltItem setSubmenu:libFiltMenu];
        [fileMenu addItem:[NSMenuItem separatorItem]];
        add_action_item(fileMenu, @"Page Setup...", @"p", NSEventModifierFlagCommand | NSEventModifierFlagShift, ACTION_PAGE_SETUP);
        add_action_item(fileMenu, @"Printer...", @"p", NSEventModifierFlagCommand, ACTION_PRINTER);
        [fileTopItem setSubmenu:fileMenu];

        // ====================================================================
        // 2. Edit Menu (Screenshot 19)
        // ====================================================================
        NSMenuItem *editTopItem = [[NSMenuItem alloc] init];
        [mainMenu addItem:editTopItem];
        NSMenu *editMenu = [[NSMenu alloc] initWithTitle:@"Edit"];
        add_action_item(editMenu, @"Undo", @"z", NSEventModifierFlagCommand, ACTION_UNDO);
        add_action_item(editMenu, @"Redo", @"z", NSEventModifierFlagCommand | NSEventModifierFlagShift, ACTION_REDO);
        [editMenu addItem:[NSMenuItem separatorItem]];
        add_std_item(editMenu, @"Cut", @selector(cut:), @"x", NSEventModifierFlagCommand);
        add_action_item(editMenu, @"Copy Settings", @"c", NSEventModifierFlagCommand, ACTION_COPY);
        add_action_item(editMenu, @"Paste Settings", @"v", NSEventModifierFlagCommand, ACTION_PASTE);
        add_action_item(editMenu, @"Delete", @"\x7F", 0, ACTION_DELETE);
        [editMenu addItem:[NSMenuItem separatorItem]];
        add_action_item(editMenu, @"Select All", @"a", NSEventModifierFlagCommand, ACTION_SELECT_ALL);
        add_action_item(editMenu, @"Select None", @"d", NSEventModifierFlagCommand, ACTION_SELECT_NONE);
        add_action_item(editMenu, @"Invert Selection", @"", 0, ACTION_INVERT_SELECTION);
        add_action_item(editMenu, @"Select Only Active Photo", @"d", NSEventModifierFlagCommand | NSEventModifierFlagShift, ACTION_SELECT_ONLY_ACTIVE);
        add_action_item(editMenu, @"Deselect Active Photo", @"/", 0, ACTION_DESELECT_ACTIVE);
        add_action_item(editMenu, @"Select Flagged Photos", @"a", NSEventModifierFlagCommand | NSEventModifierFlagOption, ACTION_SELECT_FLAGGED);
        add_action_item(editMenu, @"Deselect Unflagged Photos", @"d", NSEventModifierFlagCommand | NSEventModifierFlagOption | NSEventModifierFlagShift, ACTION_DESELECT_UNFLAGGED);
        [editMenu addItem:[NSMenuItem separatorItem]];
        add_action_item(editMenu, @"Edit Watermarks...", @"", 0, ACTION_EDIT_WATERMARKS);
        [editTopItem setSubmenu:editMenu];

        // ====================================================================
        // 3. Develop Menu (Screenshot 20)
        // ====================================================================
        NSMenuItem *devTopItem = [[NSMenuItem alloc] init];
        [mainMenu addItem:devTopItem];
        NSMenu *devMenu = [[NSMenu alloc] initWithTitle:@"Develop"];
        add_action_item(devMenu, @"New Snapshot...", @"n", NSEventModifierFlagCommand, ACTION_NEW_SNAPSHOT);
        add_action_item(devMenu, @"New Preset...", @"n", NSEventModifierFlagCommand | NSEventModifierFlagShift, ACTION_NEW_PRESET);
        add_action_item(devMenu, @"Clear History...", @"", 0, ACTION_CLEAR_HISTORY);
        [devMenu addItem:[NSMenuItem separatorItem]];
        unichar leftArrow = NSLeftArrowFunctionKey;
        unichar rightArrow = NSRightArrowFunctionKey;
        add_action_item(devMenu, @"Previous Photo", [NSString stringWithCharacters:&leftArrow length:1], NSEventModifierFlagCommand, ACTION_PREV_PHOTO);
        add_action_item(devMenu, @"Next Photo", [NSString stringWithCharacters:&rightArrow length:1], NSEventModifierFlagCommand, ACTION_NEXT_PHOTO);
        [devTopItem setSubmenu:devMenu];

        // ====================================================================
        // 4. Photo Menu (Screenshot 21)
        // ====================================================================
        NSMenuItem *photoTopItem = [[NSMenuItem alloc] init];
        [mainMenu addItem:photoTopItem];
        NSMenu *photoMenu = [[NSMenu alloc] initWithTitle:@"Photo"];
        add_action_item(photoMenu, @"Add to Quick Collection and Next", @"b", NSEventModifierFlagShift, ACTION_QUICK_COLLECTION_NEXT);
        add_action_item(photoMenu, @"Show in Finder", @"r", NSEventModifierFlagCommand, ACTION_SHOW_IN_FINDER);
        add_action_item(photoMenu, @"Go to Folder in Library", @"", 0, ACTION_GO_TO_FOLDER);
        add_action_item(photoMenu, @"Open in External Editor...", @"e", NSEventModifierFlagCommand, ACTION_EDIT_IN_EXTERNAL);
        NSMenuItem *mergeSub = [photoMenu addItemWithTitle:@"Photo Merge" action:nil keyEquivalent:@""];
        NSMenu *mergeMenu = [[NSMenu alloc] initWithTitle:@"Photo Merge"];
        add_action_item(mergeMenu, @"HDR Merge...", @"h", NSEventModifierFlagControl, ACTION_PHOTO_MERGE_HDR);
        add_action_item(mergeMenu, @"Panorama Merge...", @"m", NSEventModifierFlagControl, ACTION_PHOTO_MERGE_PANO);
        [mergeSub setSubmenu:mergeMenu];
        add_action_item(photoMenu, @"Enhance...", @"", 0, ACTION_ENHANCE_PHOTO);
        add_action_item(photoMenu, @"Create Virtual Copy", @"'", NSEventModifierFlagCommand, ACTION_CREATE_VIRTUAL_COPY);
        [photoMenu addItem:[NSMenuItem separatorItem]];
        add_action_item(photoMenu, @"Rotate Left (CCW)", @"[", NSEventModifierFlagCommand, ACTION_ROTATE_LEFT);
        add_action_item(photoMenu, @"Rotate Right (CW)", @"]", NSEventModifierFlagCommand, ACTION_ROTATE_RIGHT);
        add_action_item(photoMenu, @"Flip Horizontal", @"", 0, ACTION_FLIP_HORIZONTAL);
        add_action_item(photoMenu, @"Flip Vertical", @"", 0, ACTION_FLIP_VERTICAL);
        [photoMenu addItem:[NSMenuItem separatorItem]];
        NSMenuItem *flagSub = [photoMenu addItemWithTitle:@"Set Flag" action:nil keyEquivalent:@""];
        NSMenu *flagMenu = [[NSMenu alloc] initWithTitle:@"Set Flag"];
        add_action_item(flagMenu, @"Flag as Pick", @"p", 0, ACTION_SET_FLAG_PICK);
        add_action_item(flagMenu, @"Unflag", @"u", 0, ACTION_SET_FLAG_UNFLAG);
        add_action_item(flagMenu, @"Flag as Reject", @"x", 0, ACTION_SET_FLAG_REJECT);
        [flagSub setSubmenu:flagMenu];

        NSMenuItem *rateSub = [photoMenu addItemWithTitle:@"Set Rating" action:nil keyEquivalent:@""];
        NSMenu *rateMenu = [[NSMenu alloc] initWithTitle:@"Set Rating"];
        add_action_item(rateMenu, @"None", @"0", 0, ACTION_SET_RATING_0);
        add_action_item(rateMenu, @"★", @"1", 0, ACTION_SET_RATING_1);
        add_action_item(rateMenu, @"★★", @"2", 0, ACTION_SET_RATING_2);
        add_action_item(rateMenu, @"★★★", @"3", 0, ACTION_SET_RATING_3);
        add_action_item(rateMenu, @"★★★★", @"4", 0, ACTION_SET_RATING_4);
        add_action_item(rateMenu, @"★★★★★", @"5", 0, ACTION_SET_RATING_5);
        [rateSub setSubmenu:rateMenu];

        NSMenuItem *colorSub = [photoMenu addItemWithTitle:@"Set Color Label" action:nil keyEquivalent:@""];
        NSMenu *colorMenu = [[NSMenu alloc] initWithTitle:@"Set Color Label"];
        add_action_item(colorMenu, @"None", @"", 0, ACTION_SET_COLOR_NONE);
        add_action_item(colorMenu, @"Red", @"6", 0, ACTION_SET_COLOR_RED);
        add_action_item(colorMenu, @"Yellow", @"7", 0, ACTION_SET_COLOR_YELLOW);
        add_action_item(colorMenu, @"Green", @"8", 0, ACTION_SET_COLOR_GREEN);
        add_action_item(colorMenu, @"Blue", @"9", 0, ACTION_SET_COLOR_BLUE);
        add_action_item(colorMenu, @"Purple", @"", 0, ACTION_SET_COLOR_PURPLE);
        [colorSub setSubmenu:colorMenu];

        [photoMenu addItem:[NSMenuItem separatorItem]];
        add_action_item(photoMenu, @"Save Metadata to File", @"s", NSEventModifierFlagCommand, ACTION_SAVE_METADATA);
        add_action_item(photoMenu, @"Read Metadata from File", @"", 0, ACTION_READ_METADATA);
        add_action_item(photoMenu, @"Update DNG Preview & Metadata", @"", 0, ACTION_UPDATE_DNG_PREVIEW);
        [photoMenu addItem:[NSMenuItem separatorItem]];
        add_action_item(photoMenu, @"Remove Photo...", @"", 0, ACTION_REMOVE_PHOTO);
        add_action_item(photoMenu, @"Remove Selected Photos...", @"\x7F", NSEventModifierFlagOption, ACTION_REMOVE_SELECTED_PHOTOS);
        add_action_item(photoMenu, @"Delete Rejected Photos...", @"\x7F", NSEventModifierFlagCommand, ACTION_DELETE_REJECTED);
        [photoTopItem setSubmenu:photoMenu];

        // ====================================================================
        // 5. Settings Menu (Screenshot 22)
        // ====================================================================
        NSMenuItem *setTopItem = [[NSMenuItem alloc] init];
        [mainMenu addItem:setTopItem];
        NSMenu *setMenu = [[NSMenu alloc] initWithTitle:@"Settings"];
        add_action_item(setMenu, @"Reset All Settings", @"r", NSEventModifierFlagCommand | NSEventModifierFlagShift, ACTION_RESET_ALL_SETTINGS);
        add_action_item(setMenu, @"Update to Current Process Version", @"", 0, ACTION_UPDATE_PROCESS_VERSION);
        [setMenu addItem:[NSMenuItem separatorItem]];
        add_action_item(setMenu, @"Copy Settings...", @"c", NSEventModifierFlagCommand | NSEventModifierFlagShift, ACTION_COPY_SETTINGS);
        add_action_item(setMenu, @"Paste Settings", @"v", NSEventModifierFlagCommand | NSEventModifierFlagShift, ACTION_PASTE_SETTINGS);
        add_action_item(setMenu, @"Paste Settings from Previous", @"v", NSEventModifierFlagCommand | NSEventModifierFlagOption, ACTION_PASTE_FROM_PREVIOUS);
        add_action_item(setMenu, @"Sync Settings...", @"s", NSEventModifierFlagCommand | NSEventModifierFlagShift, ACTION_SYNC_SETTINGS);
        [setMenu addItem:[NSMenuItem separatorItem]];
        add_action_item(setMenu, @"Auto Settings", @"u", NSEventModifierFlagCommand, ACTION_AUTO_SETTINGS);
        add_action_item(setMenu, @"Auto White Balance", @"u", NSEventModifierFlagCommand | NSEventModifierFlagShift, ACTION_AUTO_WHITE_BALANCE);
        add_action_item(setMenu, @"Convert to Black & White", @"v", 0, ACTION_CONVERT_BW);
        add_action_item(setMenu, @"Toggle HDR Edit Mode", @"", 0, ACTION_TOGGLE_HDR);
        add_action_item(setMenu, @"Reset Crop", @"", 0, ACTION_RESET_CROP);
        add_action_item(setMenu, @"Constrain Aspect Ratio", @"a", NSEventModifierFlagShift, ACTION_CONSTRAIN_ASPECT);
        add_action_item(setMenu, @"Sync Crop to Library...", @"", 0, ACTION_SYNC_CROP_TO_LIBRARY);
        add_action_item(setMenu, @"Swap Before and After Settings", @"", 0, ACTION_SWAP_BEFORE_AFTER);
        [setTopItem setSubmenu:setMenu];

        // ====================================================================
        // 6. Tools Menu (Screenshot 23)
        // ====================================================================
        NSMenuItem *toolsTopItem = [[NSMenuItem alloc] init];
        [mainMenu addItem:toolsTopItem];
        NSMenu *toolsMenu = [[NSMenu alloc] initWithTitle:@"Tools"];
        add_action_item(toolsMenu, @"Crop", @"r", 0, ACTION_TOOL_CROP);
        add_action_item(toolsMenu, @"Red Eye", @"", 0, ACTION_TOOL_RED_EYE);
        add_action_item(toolsMenu, @"Remove", @"q", 0, ACTION_TOOL_REMOVE);
        add_action_item(toolsMenu, @"Masking", @"k", NSEventModifierFlagShift, ACTION_TOOL_MASKING);
        add_action_item(toolsMenu, @"Guided Upright", @"t", NSEventModifierFlagShift, ACTION_TOOL_GUIDED_UPRIGHT);
        add_action_item(toolsMenu, @"White Balance Selector", @"w", 0, ACTION_TOOL_WB_SELECTOR);
        add_action_item(toolsMenu, @"Point Color Selector", @"p", NSEventModifierFlagControl | NSEventModifierFlagShift, ACTION_TOOL_POINT_COLOR);
        add_action_item(toolsMenu, @"Target Adjustment", @"", 0, ACTION_TOOL_TARGET_ADJUST);
        [toolsTopItem setSubmenu:toolsMenu];

        // ====================================================================
        // 7. View Menu (Screenshot 24)
        // ====================================================================
        NSMenuItem *viewTopItem = [[NSMenuItem alloc] init];
        [mainMenu addItem:viewTopItem];
        NSMenu *viewMenu = [[NSMenu alloc] initWithTitle:@"View"];
        add_action_item(viewMenu, @"Hide Toolbar", @"t", 0, ACTION_HIDE_TOOLBAR);
        add_action_item(viewMenu, @"Cycle Loupe View", @"i", 0, ACTION_CYCLE_LOUPE);
        add_action_item(viewMenu, @"Toggle Zoom View", @"z", 0, ACTION_TOGGLE_ZOOM);
        add_action_item(viewMenu, @"Zoom In", @"=", NSEventModifierFlagCommand, ACTION_ZOOM_IN);
        add_action_item(viewMenu, @"Zoom Out", @"-", NSEventModifierFlagCommand, ACTION_ZOOM_OUT);
        add_action_item(viewMenu, @"Zoom To 100%", @"0", NSEventModifierFlagCommand | NSEventModifierFlagOption, ACTION_ZOOM_100);
        [viewMenu addItem:[NSMenuItem separatorItem]];
        add_action_item(viewMenu, @"Loupe", @"d", 0, ACTION_MODULE_DEVELOP);
        add_action_item(viewMenu, @"Open in Reference View", @"r", NSEventModifierFlagShift, ACTION_OPEN_REFERENCE_VIEW);
        add_action_item(viewMenu, @"Before / After", @"y", 0, ACTION_BEFORE_AFTER);
        add_action_item(viewMenu, @"Soft Proofing", @"s", 0, ACTION_SOFT_PROOFING);
        add_action_item(viewMenu, @"Show Clipping", @"j", 0, ACTION_SHOW_CLIPPING);
        [viewMenu addItem:[NSMenuItem separatorItem]];
        NSMenuItem *goToLib = [viewMenu addItemWithTitle:@"Go to Library:" action:nil keyEquivalent:@""];
        NSMenu *goToLibMenu = [[NSMenu alloc] initWithTitle:@"Go to Library:"];
        add_action_item(goToLibMenu, @"Grid", @"g", 0, ACTION_VIEW_GRID);
        add_action_item(goToLibMenu, @"Loupe", @"e", 0, ACTION_VIEW_LOUPE);
        add_action_item(goToLibMenu, @"Compare", @"c", 0, ACTION_VIEW_COMPARE);
        add_action_item(goToLibMenu, @"Survey", @"n", 0, ACTION_VIEW_SURVEY);
        add_action_item(goToLibMenu, @"People", @"o", 0, ACTION_VIEW_PEOPLE);
        [goToLib setSubmenu:goToLibMenu];
        [viewTopItem setSubmenu:viewMenu];

        // ====================================================================
        // 8. Window Menu (Screenshot 25)
        // ====================================================================
        NSMenuItem *winTopItem = [[NSMenuItem alloc] init];
        [mainMenu addItem:winTopItem];
        NSMenu *winMenu = [[NSMenu alloc] initWithTitle:@"Window"];
        add_std_item(winMenu, @"Close Window", @selector(performClose:), @"w", NSEventModifierFlagCommand);
        add_std_item(winMenu, @"Minimize", @selector(performMiniaturize:), @"m", NSEventModifierFlagCommand);
        add_std_item(winMenu, @"Zoom", @selector(performZoom:), @"", 0);
        add_action_item(winMenu, @"Impromptu Slideshow", @"\r", NSEventModifierFlagCommand, ACTION_IMPROMPTU_SLIDESHOW);
        add_std_item(winMenu, @"Enter Full Screen", @selector(toggleFullScreen:), @"f", NSEventModifierFlagCommand | NSEventModifierFlagControl);
        [winMenu addItem:[NSMenuItem separatorItem]];
        add_action_item(winMenu, @"Lights Out", @"l", 0, ACTION_LIGHTS_OUT);
        [winMenu addItem:[NSMenuItem separatorItem]];
        NSMenuItem *panelsItem = [winMenu addItemWithTitle:@"Panels" action:nil keyEquivalent:@""];
        NSMenu *panelsMenu = [[NSMenu alloc] initWithTitle:@"Panels"];
        add_action_item(panelsMenu, @"Toggle Left Panel", @"\t", 0, ACTION_TOGGLE_LEFT_PANEL);
        add_action_item(panelsMenu, @"Toggle Right Panel", @"\t", NSEventModifierFlagShift, ACTION_TOGGLE_RIGHT_PANEL);
        add_action_item(panelsMenu, @"Toggle Filmstrip", @"", 0, ACTION_TOGGLE_FILMSTRIP);
        [panelsItem setSubmenu:panelsMenu];
        [winMenu addItem:[NSMenuItem separatorItem]];
        NSMenuItem *modItem = [winMenu addItemWithTitle:@"Modules:" action:nil keyEquivalent:@""];
        NSMenu *modMenu = [[NSMenu alloc] initWithTitle:@"Modules:"];
        add_action_item(modMenu, @"Library", @"1", NSEventModifierFlagCommand | NSEventModifierFlagOption, ACTION_MODULE_LIBRARY);
        add_action_item(modMenu, @"Develop", @"2", NSEventModifierFlagCommand | NSEventModifierFlagOption, ACTION_MODULE_DEVELOP);
        add_action_item(modMenu, @"Map", @"3", NSEventModifierFlagCommand | NSEventModifierFlagOption, ACTION_MODULE_MAP);
        add_action_item(modMenu, @"Book", @"4", NSEventModifierFlagCommand | NSEventModifierFlagOption, ACTION_MODULE_BOOK);
        add_action_item(modMenu, @"Slideshow", @"5", NSEventModifierFlagCommand | NSEventModifierFlagOption, ACTION_MODULE_SLIDESHOW);
        add_action_item(modMenu, @"Print", @"6", NSEventModifierFlagCommand | NSEventModifierFlagOption, ACTION_MODULE_PRINT);
        add_action_item(modMenu, @"Web", @"7", NSEventModifierFlagCommand | NSEventModifierFlagOption, ACTION_MODULE_WEB);
        [modItem setSubmenu:modMenu];
        [app setWindowsMenu:winMenu];
        [winTopItem setSubmenu:winMenu];

        // ====================================================================
        // 9. Help Menu (Screenshot 26)
        // ====================================================================
        NSMenuItem *helpTopItem = [[NSMenuItem alloc] init];
        [mainMenu addItem:helpTopItem];
        NSMenu *helpMenu = [[NSMenu alloc] initWithTitle:@"Help"];
        add_action_item(helpMenu, @"What's New...", @"", 0, ACTION_WHATS_NEW);
        add_action_item(helpMenu, @"Check for Updates...", @"", 0, ACTION_CHECK_UPDATES);
        add_action_item(helpMenu, @"LightRaw Help...", @"?", NSEventModifierFlagCommand | NSEventModifierFlagShift, ACTION_HELP_DOCUMENTATION);
        add_action_item(helpMenu, @"System Info...", @"", 0, ACTION_SYSTEM_INFO);
        add_action_item(helpMenu, @"Develop Module Shortcuts...", @"/", NSEventModifierFlagCommand, ACTION_SHORTCUTS);
        [app setHelpMenu:helpMenu];
        [helpTopItem setSubmenu:helpMenu];

        [app setMainMenu:mainMenu];
    }
    });
}

// ============================================================================
// Native Metal / CoreImage Linear Scene-Referred Camera RAW & Image Decoder
// ============================================================================

static CIContext *shared_linear_ci_context(void) {
    static CIContext *ctx = nil;
    static dispatch_once_t onceToken;
    dispatch_once(&onceToken, ^{
        CGColorSpaceRef linCS = CGColorSpaceCreateWithName(kCGColorSpaceExtendedLinearSRGB);
        NSDictionary *opts = @{
            kCIContextWorkingColorSpace: (__bridge id)linCS,
            kCIContextOutputColorSpace: (__bridge id)linCS,
            kCIContextCacheIntermediates: @NO
        };
        ctx = [CIContext contextWithOptions:opts];
        CGColorSpaceRelease(linCS);
    });
    return ctx;
}

float *lightraw_decode_image_linear(
    const char *path_utf8,
    uint32_t max_dim,
    int is_raw_hint,
    int draft_mode,
    uint32_t *out_w,
    uint32_t *out_h,
    uint32_t *out_native_w,
    uint32_t *out_native_h,
    float *out_as_shot_temp,
    float *out_as_shot_tint,
    float *out_baseline_ev,
    int *out_is_scene_referred
) {
    if (!path_utf8 || !out_w || !out_h) {
        return NULL;
    }
    @autoreleasepool {
        NSString *nsPath = [NSString stringWithUTF8String:path_utf8];
        if (!nsPath) return NULL;
        NSURL *url = [NSURL fileURLWithPath:nsPath];
        if (!url) return NULL;

        CIImage *ciOut = nil;
        float asTemp = 5500.0f;
        float asTint = 0.0f;
        float baseEv = 0.0f;
        int sceneRef = 0;
        uint32_t natW = 0;
        uint32_t natH = 0;

        if (is_raw_hint) {
            if (@available(macOS 12.0, *)) {
                CIRAWFilter *rf = [CIRAWFilter filterWithImageURL:url];
                if (rf != nil) {
                    asTemp = rf.neutralTemperature;
                    asTint = rf.neutralTint;
                    baseEv = rf.baselineExposure;
                    CGSize ns = rf.nativeSize;
                    CGImagePropertyOrientation orient = rf.orientation;
                    if (orient == kCGImagePropertyOrientationLeft ||
                        orient == kCGImagePropertyOrientationRight ||
                        orient == kCGImagePropertyOrientationLeftMirrored ||
                        orient == kCGImagePropertyOrientationRightMirrored) {
                        natW = (uint32_t)llround(ns.height);
                        natH = (uint32_t)llround(ns.width);
                    } else {
                        natW = (uint32_t)llround(ns.width);
                        natH = (uint32_t)llround(ns.height);
                    }

                    // Pure linear unclipped sensor demosaicing (no baked Apple S-curve)
                    rf.boostAmount = 0.0f;
                    rf.boostShadowAmount = 0.0f;
                    rf.exposure = 0.0f;
                    rf.gamutMappingEnabled = NO;
                    rf.draftModeEnabled = (draft_mode != 0) ? YES : NO;
                    if (rf.supportedDecoderVersions != nil && rf.supportedDecoderVersions.count > 0) {
                        rf.decoderVersion = rf.supportedDecoderVersions.lastObject;
                    }
                    if (rf.sharpnessSupported) {
                        rf.sharpnessAmount = 0.5f;
                    }
                    if (rf.lensCorrectionSupported) {
                        rf.lensCorrectionEnabled = YES;
                    }
                    if (rf.moireReductionSupported) {
                        rf.moireReductionAmount = 0.35f;
                    }

                    double longest = fmax((double)natW, (double)natH);
                    if (max_dim > 0 && longest > (double)max_dim) {
                        float scale = (float)((double)max_dim / longest);
                        if (scale < 0.02f) scale = 0.02f;
                        rf.scaleFactor = scale;
                    } else {
                        rf.scaleFactor = 1.0f;
                    }

                    ciOut = rf.outputImage;
                    if (ciOut != nil) {
                        sceneRef = 1;
                    }
                }
            }
        }

        if (ciOut == nil) {
            // Standard RGB / HEIC / TIFF / PNG / JPEG or fallback via CIImage + orientation
            NSDictionary *ciOpts = @{ kCIImageApplyOrientationProperty: @YES };
            CIImage *baseImg = [CIImage imageWithContentsOfURL:url options:ciOpts];
            if (baseImg == nil) {
                return NULL;
            }
            CGRect ext0 = baseImg.extent;
            natW = (uint32_t)llround(ext0.size.width);
            natH = (uint32_t)llround(ext0.size.height);
            double longest = fmax(ext0.size.width, ext0.size.height);
            if (max_dim > 0 && longest > (double)max_dim) {
                double scale = (double)max_dim / longest;
                ciOut = [baseImg imageByApplyingTransform:CGAffineTransformMakeScale(scale, scale)];
            } else {
                ciOut = baseImg;
            }
            sceneRef = 0;
        }

        CGRect extent = CGRectIntegral(ciOut.extent);
        if (CGRectIsEmpty(extent) || extent.size.width < 1.0 || extent.size.height < 1.0) {
            return NULL;
        }
        uint32_t w = (uint32_t)extent.size.width;
        uint32_t h = (uint32_t)extent.size.height;
        size_t numPixels = (size_t)w * (size_t)h;

        float *rgbaBuf = (float *)malloc(numPixels * 4 * sizeof(float));
        if (!rgbaBuf) {
            return NULL;
        }

        CGColorSpaceRef linCS = CGColorSpaceCreateWithName(kCGColorSpaceExtendedLinearSRGB);
        CIContext *ctx = shared_linear_ci_context();
        [ctx render:ciOut
           toBitmap:rgbaBuf
           rowBytes:(size_t)w * 4 * sizeof(float)
             bounds:extent
             format:kCIFormatRGBAf
         colorSpace:linCS];
        CGColorSpaceRelease(linCS);

        float *rgbBuf = (float *)malloc(numPixels * 3 * sizeof(float));
        if (!rgbBuf) {
            free(rgbaBuf);
            return NULL;
        }
        for (size_t i = 0; i < numPixels; i++) {
            float r = rgbaBuf[i * 4 + 0];
            float g = rgbaBuf[i * 4 + 1];
            float b = rgbaBuf[i * 4 + 2];
            rgbBuf[i * 3 + 0] = isfinite(r) ? fmaxf(r, 0.0f) : 0.0f;
            rgbBuf[i * 3 + 1] = isfinite(g) ? fmaxf(g, 0.0f) : 0.0f;
            rgbBuf[i * 3 + 2] = isfinite(b) ? fmaxf(b, 0.0f) : 0.0f;
        }
        free(rgbaBuf);

        *out_w = w;
        *out_h = h;
        if (out_native_w) *out_native_w = (natW > 0) ? natW : w;
        if (out_native_h) *out_native_h = (natH > 0) ? natH : h;
        if (out_as_shot_temp) *out_as_shot_temp = (asTemp > 1000.0f && asTemp < 50000.0f) ? asTemp : 5500.0f;
        if (out_as_shot_tint) *out_as_shot_tint = (isfinite(asTint)) ? asTint : 0.0f;
        if (out_baseline_ev) *out_baseline_ev = (isfinite(baseEv)) ? baseEv : 0.0f;
        if (out_is_scene_referred) *out_is_scene_referred = sceneRef;

        return rgbBuf;
    }
}

void lightraw_free_float_buffer(float *ptr) {
    if (ptr) {
        free(ptr);
    }
}

// ============================================================================
// Apple Vision Framework Real Face, Eye-Openness & Head Angle Detection
// ============================================================================

typedef struct {
    float x;       // Normalized [0..1] top-left X
    float y;       // Normalized [0..1] top-left Y
    float w;       // Normalized [0..1] width
    float h;       // Normalized [0..1] height
    float quality; // 0..100
    float eyes_open_score; // 0..100 (>= 45 = open, < 45 = closed/blink)
    float yaw_deg; // Head yaw angle in degrees
} LightRawFaceInfo;

int lightraw_detect_faces_rgba(
    const uint8_t *rgba_ptr,
    uint32_t width,
    uint32_t height,
    LightRawFaceInfo *out_faces,
    int max_faces
) {
    if (!rgba_ptr || width < 16 || height < 16 || !out_faces || max_faces <= 0) {
        return 0;
    }
    @autoreleasepool {
        CGColorSpaceRef cs = CGColorSpaceCreateDeviceRGB();
        CGDataProviderRef provider = CGDataProviderCreateWithData(NULL, rgba_ptr, (size_t)width * (size_t)height * 4, NULL);
        CGImageRef cgImg = CGImageCreate(
            width, height, 8, 32, (size_t)width * 4, cs,
            kCGImageAlphaPremultipliedLast | kCGBitmapByteOrderDefault,
            provider, NULL, false, kCGRenderingIntentDefault
        );
        CGDataProviderRelease(provider);
        CGColorSpaceRelease(cs);
        if (!cgImg) return 0;

        VNImageRequestHandler *handler = [[VNImageRequestHandler alloc] initWithCGImage:cgImg options:@{}];
        VNDetectFaceLandmarksRequest *req = [[VNDetectFaceLandmarksRequest alloc] init];
        NSError *err = nil;
        BOOL ok = [handler performRequests:@[req] error:&err];
        CGImageRelease(cgImg);
        if (!ok || err != nil) {
            return 0;
        }

        NSArray<VNFaceObservation *> *results = req.results;
        int count = 0;
        for (VNFaceObservation *obs in results) {
            if (count >= max_faces) break;
            CGRect bb = obs.boundingBox; // Vision origin is bottom-left!
            LightRawFaceInfo info;
            info.x = (float)bb.origin.x;
            info.y = (float)(1.0 - bb.origin.y - bb.size.height);
            info.w = (float)bb.size.width;
            info.h = (float)bb.size.height;
            info.quality = obs.confidence * 100.0f;
            info.yaw_deg = obs.yaw ? (float)(obs.yaw.doubleValue * 180.0 / M_PI) : 0.0f;

            // Compute eye aspect ratio from Vision landmarks to detect blinks
            float eyeScore = 88.0f;
            VNFaceLandmarks2D *lm = obs.landmarks;
            if (lm != nil && lm.leftEye != nil && lm.rightEye != nil &&
                lm.leftEye.pointCount >= 6 && lm.rightEye.pointCount >= 6) {
                const CGPoint *lp = lm.leftEye.normalizedPoints;
                const CGPoint *rp = lm.rightEye.normalizedPoints;
                double lMinX = 1.0, lMaxX = 0.0, lMinY = 1.0, lMaxY = 0.0;
                for (NSUInteger k = 0; k < lm.leftEye.pointCount; k++) {
                    if (lp[k].x < lMinX) lMinX = lp[k].x;
                    if (lp[k].x > lMaxX) lMaxX = lp[k].x;
                    if (lp[k].y < lMinY) lMinY = lp[k].y;
                    if (lp[k].y > lMaxY) lMaxY = lp[k].y;
                }
                double rMinX = 1.0, rMaxX = 0.0, rMinY = 1.0, rMaxY = 0.0;
                for (NSUInteger k = 0; k < lm.rightEye.pointCount; k++) {
                    if (rp[k].x < rMinX) rMinX = rp[k].x;
                    if (rp[k].x > rMaxX) rMaxX = rp[k].x;
                    if (rp[k].y < rMinY) rMinY = rp[k].y;
                    if (rp[k].y > rMaxY) rMaxY = rp[k].y;
                }
                double lEar = (lMaxY - lMinY) / fmax(lMaxX - lMinX, 1e-4);
                double rEar = (rMaxY - rMinY) / fmax(rMaxX - rMinX, 1e-4);
                double ear = 0.5 * (lEar + rEar);
                // Typical open eye aspect ratio ~0.22..0.38; closed blink < 0.14
                double mapped = (ear - 0.08) / 0.22 * 100.0;
                if (mapped < 5.0) mapped = 5.0;
                if (mapped > 100.0) mapped = 100.0;
                eyeScore = (float)mapped;
            }
            info.eyes_open_score = eyeScore;
            out_faces[count++] = info;
        }
        return count;
    }
}

// ============================================================================
// Native macOS Page Setup, Print Dialog & Disk Volume Space Query
// ============================================================================

void lightraw_run_page_setup(void) {
    @autoreleasepool {
        NSPageLayout *pageLayout = [NSPageLayout pageLayout];
        [pageLayout runModal];
    }
}

int lightraw_print_rgba_sheet(const uint8_t *rgba_ptr, uint32_t width, uint32_t height, const char *job_title_utf8) {
    if (!rgba_ptr || width == 0 || height == 0) return 0;
    @autoreleasepool {
        NSBitmapImageRep *rep = [[NSBitmapImageRep alloc]
            initWithBitmapDataPlanes:NULL
                          pixelsWide:(NSInteger)width
                          pixelsHigh:(NSInteger)height
                       bitsPerSample:8
                     samplesPerPixel:4
                            hasAlpha:YES
                            isPlanar:NO
                      colorSpaceName:NSDeviceRGBColorSpace
                         bytesPerRow:(NSInteger)width * 4
                        bitsPerPixel:32];
        if (!rep || !rep.bitmapData) return 0;
        memcpy(rep.bitmapData, rgba_ptr, (size_t)width * (size_t)height * 4);

        NSImage *img = [[NSImage alloc] initWithSize:NSMakeSize(width, height)];
        [img addRepresentation:rep];

        NSPrintInfo *printInfo = [[NSPrintInfo sharedPrintInfo] copy];
        [printInfo setHorizontalPagination:NSPrintingPaginationModeFit];
        [printInfo setVerticalPagination:NSPrintingPaginationModeFit];
        [printInfo setHorizontallyCentered:YES];
        [printInfo setVerticallyCentered:YES];
        if (width > height) {
            [printInfo setOrientation:NSPaperOrientationLandscape];
        } else {
            [printInfo setOrientation:NSPaperOrientationPortrait];
        }

        NSRect viewFrame = NSMakeRect(0, 0, printInfo.paperSize.width - printInfo.leftMargin - printInfo.rightMargin,
                                      printInfo.paperSize.height - printInfo.topMargin - printInfo.bottomMargin);
        NSImageView *imgView = [[NSImageView alloc] initWithFrame:viewFrame];
        imgView.image = img;
        imgView.imageScaling = NSImageScaleProportionallyUpOrDown;

        NSPrintOperation *op = [NSPrintOperation printOperationWithView:imgView printInfo:printInfo];
        if (job_title_utf8) {
            op.jobTitle = [NSString stringWithUTF8String:job_title_utf8];
        }
        op.showsPrintPanel = YES;
        op.showsProgressPanel = YES;
        return [op runOperation] ? 1 : 0;
    }
}

int lightraw_query_disk_space(const char *path_utf8, uint64_t *out_free_bytes, uint64_t *out_total_bytes, char *out_vol_name, size_t vol_name_cap) {
    if (!path_utf8) return 0;
    @autoreleasepool {
        NSString *nsPath = [NSString stringWithUTF8String:path_utf8];
        if (!nsPath) return 0;
        NSDictionary *attrs = [[NSFileManager defaultManager] attributesOfFileSystemForPath:nsPath error:nil];
        if (!attrs) return 0;
        NSNumber *freeNum = attrs[NSFileSystemFreeSize];
        NSNumber *totalNum = attrs[NSFileSystemSize];
        if (out_free_bytes && freeNum) *out_free_bytes = freeNum.unsignedLongLongValue;
        if (out_total_bytes && totalNum) *out_total_bytes = totalNum.unsignedLongLongValue;

        if (out_vol_name && vol_name_cap > 1) {
            NSURL *url = [NSURL fileURLWithPath:nsPath];
            NSString *volName = nil;
            [url getResourceValue:&volName forKey:NSURLVolumeNameKey error:nil];
            if (!volName || volName.length == 0) {
                volName = @"Macintosh HD";
            }
            const char *cstr = [volName UTF8String];
            if (cstr) {
                strncpy(out_vol_name, cstr, vol_name_cap - 1);
                out_vol_name[vol_name_cap - 1] = '\0';
            }
        }
        return 1;
    }
}

// ============================================================================
// LightRaw Native Apple Neural Engine (ANE) & Metal AI Hardware Bridge
// ============================================================================

int lightraw_ai_hardware_info(char *out_desc, size_t max_len) {
    if (!out_desc || max_len == 0) return 0;
    @autoreleasepool {
        id<MTLDevice> dev = MTLCreateSystemDefaultDevice();
        NSString *gpuName = dev ? dev.name : @"Apple Silicon GPU";
        NSString *info = [NSString stringWithFormat:@"Apple Neural Engine (16-core ANE) + %@", gpuName];
        strncpy(out_desc, [info UTF8String], max_len - 1);
        out_desc[max_len - 1] = '\0';
        return 1;
    }
}

int lightraw_ai_segment_subject_rgba(
    const uint8_t *rgba_ptr,
    uint32_t width,
    uint32_t height,
    uint8_t *out_mask,
    uint32_t mask_w,
    uint32_t mask_h
) {
    if (!rgba_ptr || width < 16 || height < 16 || !out_mask || mask_w == 0 || mask_h == 0) {
        return 0;
    }
    @autoreleasepool {
        CGColorSpaceRef cs = CGColorSpaceCreateDeviceRGB();
        CGDataProviderRef provider = CGDataProviderCreateWithData(NULL, rgba_ptr, (size_t)width * (size_t)height * 4, NULL);
        CGImageRef cgImg = CGImageCreate(
            width, height, 8, 32, (size_t)width * 4, cs,
            (CGBitmapInfo)kCGImageAlphaPremultipliedLast,
            provider, NULL, false, kCGRenderingIntentDefault
        );
        CGDataProviderRelease(provider);
        CGColorSpaceRelease(cs);
        if (!cgImg) return 0;

        VNImageRequestHandler *handler = [[VNImageRequestHandler alloc] initWithCGImage:cgImg options:@{}];
        CVPixelBufferRef maskPb = NULL;
        NSError *err = nil;

        // 1. Primary: macOS 14+ Foreground Instance Mask (People, Animals, Vehicles, Objects)
        if (@available(macOS 14.0, *)) {
            VNGenerateForegroundInstanceMaskRequest *fgReq = [[VNGenerateForegroundInstanceMaskRequest alloc] init];
            if ([handler performRequests:@[fgReq] error:&err] && fgReq.results.count > 0) {
                VNInstanceMaskObservation *fgObs = fgReq.results.firstObject;
                maskPb = [fgObs generateMaskForInstances:fgObs.allInstances error:&err];
            }
        }

        // 2. Fallback: macOS 12+ Accurate Person Segmentation Request
        if (!maskPb) {
            VNGeneratePersonSegmentationRequest *personReq = [[VNGeneratePersonSegmentationRequest alloc] init];
            personReq.qualityLevel = VNGeneratePersonSegmentationRequestQualityLevelAccurate;
            personReq.outputPixelFormat = kCVPixelFormatType_OneComponent8;
            if ([handler performRequests:@[personReq] error:&err] && personReq.results.count > 0) {
                VNPixelBufferObservation *pObs = personReq.results.firstObject;
                maskPb = CVPixelBufferRetain(pObs.pixelBuffer);
            }
        }

        CGImageRelease(cgImg);
        if (!maskPb) return 0;

        CVPixelBufferLockBaseAddress(maskPb, kCVPixelBufferLock_ReadOnly);
        size_t pbW = CVPixelBufferGetWidth(maskPb);
        size_t pbH = CVPixelBufferGetHeight(maskPb);
        size_t bpr = CVPixelBufferGetBytesPerRow(maskPb);
        OSType fmt = CVPixelBufferGetPixelFormatType(maskPb);
        const uint8_t *base = (const uint8_t *)CVPixelBufferGetBaseAddress(maskPb);

        for (uint32_t my = 0; my < mask_h; my++) {
            float sy_norm = (float)my / (float)mask_h;
            size_t py = (size_t)(sy_norm * (float)(pbH - 1));
            if (py >= pbH) py = pbH - 1;
            const uint8_t *row_ptr = base + py * bpr;

            for (uint32_t mx = 0; mx < mask_w; mx++) {
                float sx_norm = (float)mx / (float)mask_w;
                size_t px = (size_t)(sx_norm * (float)(pbW - 1));
                if (px >= pbW) px = pbW - 1;

                uint8_t val = 0;
                if (fmt == kCVPixelFormatType_OneComponent32Float) {
                    const float *f_row = (const float *)row_ptr;
                    float f_val = f_row[px];
                    val = (uint8_t)(fminf(fmaxf(f_val * 255.0f, 0.0f), 255.0f));
                } else {
                    val = row_ptr[px];
                }
                out_mask[my * mask_w + mx] = val;
            }
        }

        CVPixelBufferUnlockBaseAddress(maskPb, kCVPixelBufferLock_ReadOnly);
        CVPixelBufferRelease(maskPb);
        return 1;
    }
}

static int is_point_in_polygon_helper(float px, float py, const CGPoint *pts, size_t count) {
    if (count < 3 || !pts) return 0;
    int inside = 0;
    for (size_t i = 0, j = count - 1; i < count; j = i++) {
        float xi = (float)pts[i].x, yi = (float)pts[i].y;
        float xj = (float)pts[j].x, yj = (float)pts[j].y;
        if (((yi > py) != (yj > py)) && (px < (xj - xi) * (py - yi) / (yj - yi) + xi)) {
            inside = !inside;
        }
    }
    return inside;
}

static void rasterize_feature_region(
    uint8_t *mask, uint32_t mw, uint32_t mh,
    const CGPoint *norm_pts, size_t count,
    CGRect face_bb, uint8_t part_id
) {
    if (count < 3 || !norm_pts || !mask) return;
    CGPoint *img_pts = (CGPoint *)malloc(sizeof(CGPoint) * count);
    if (!img_pts) return;
    float min_x = 1.0f, max_x = 0.0f, min_y = 1.0f, max_y = 0.0f;
    for (size_t i = 0; i < count; i++) {
        float x = (float)(face_bb.origin.x + norm_pts[i].x * face_bb.size.width);
        float y = (float)(1.0 - (face_bb.origin.y + norm_pts[i].y * face_bb.size.height));
        img_pts[i] = CGPointMake(x, y);
        if (x < min_x) min_x = x;
        if (x > max_x) max_x = x;
        if (y < min_y) min_y = y;
        if (y > max_y) max_y = y;
    }

    uint32_t bx0 = (uint32_t)fmaxf(0.0f, floorf(min_x * (float)mw));
    uint32_t bx1 = (uint32_t)fminf((float)mw, ceilf(max_x * (float)mw));
    uint32_t by0 = (uint32_t)fmaxf(0.0f, floorf(min_y * (float)mh));
    uint32_t by1 = (uint32_t)fminf((float)mh, ceilf(max_y * (float)mh));

    for (uint32_t y = by0; y < by1; y++) {
        float py = ((float)y + 0.5f) / (float)mh;
        for (uint32_t x = bx0; x < bx1; x++) {
            float px = ((float)x + 0.5f) / (float)mw;
            if (is_point_in_polygon_helper(px, py, img_pts, count)) {
                mask[y * mw + x] = part_id;
            }
        }
    }
    free(img_pts);
}

int lightraw_ai_segment_people_parts_rgba(
    const uint8_t *rgba_ptr,
    uint32_t width,
    uint32_t height,
    uint8_t *out_parts_mask,
    uint32_t mask_w,
    uint32_t mask_h
) {
    if (!rgba_ptr || width < 16 || height < 16 || !out_parts_mask || mask_w == 0 || mask_h == 0) {
        return 0;
    }
    @autoreleasepool {
        CGColorSpaceRef cs = CGColorSpaceCreateDeviceRGB();
        CGDataProviderRef provider = CGDataProviderCreateWithData(NULL, rgba_ptr, (size_t)width * (size_t)height * 4, NULL);
        CGImageRef cgImg = CGImageCreate(
            width, height, 8, 32, (size_t)width * 4, cs,
            (CGBitmapInfo)kCGImageAlphaPremultipliedLast,
            provider, NULL, false, kCGRenderingIntentDefault
        );
        CGDataProviderRelease(provider);
        CGColorSpaceRelease(cs);
        if (!cgImg) return 0;

        memset(out_parts_mask, 0, (size_t)mask_w * (size_t)mask_h);

        VNImageRequestHandler *handler = [[VNImageRequestHandler alloc] initWithCGImage:cgImg options:@{}];
        VNGeneratePersonSegmentationRequest *personReq = [[VNGeneratePersonSegmentationRequest alloc] init];
        personReq.qualityLevel = VNGeneratePersonSegmentationRequestQualityLevelAccurate;
        personReq.outputPixelFormat = kCVPixelFormatType_OneComponent8;

        VNDetectFaceLandmarksRequest *faceReq = [[VNDetectFaceLandmarksRequest alloc] init];

        NSError *err = nil;
        [handler performRequests:@[personReq, faceReq] error:&err];
        CGImageRelease(cgImg);

        // 1. Fill base person body segmentation (code 6: Body/Clothes/Hair)
        if (personReq.results.count > 0) {
            VNPixelBufferObservation *pObs = personReq.results.firstObject;
            CVPixelBufferRef pb = pObs.pixelBuffer;
            if (pb) {
                CVPixelBufferLockBaseAddress(pb, kCVPixelBufferLock_ReadOnly);
                size_t pbW = CVPixelBufferGetWidth(pb);
                size_t pbH = CVPixelBufferGetHeight(pb);
                size_t bpr = CVPixelBufferGetBytesPerRow(pb);
                const uint8_t *base = (const uint8_t *)CVPixelBufferGetBaseAddress(pb);

                for (uint32_t my = 0; my < mask_h; my++) {
                    size_t py = (size_t)(((float)my / (float)mask_h) * (float)(pbH - 1));
                    const uint8_t *row = base + py * bpr;
                    for (uint32_t mx = 0; mx < mask_w; mx++) {
                        size_t px = (size_t)(((float)mx / (float)mask_w) * (float)(pbW - 1));
                        if (row[px] > 100) {
                            out_parts_mask[my * mask_w + mx] = 6; // Entire person / body
                        }
                    }
                }
                CVPixelBufferUnlockBaseAddress(pb, kCVPixelBufferLock_ReadOnly);
            }
        }

        // 2. Overlay precise face landmark parts
        for (VNFaceObservation *face in faceReq.results) {
            CGRect bb = face.boundingBox;
            VNFaceLandmarks2D *lm = face.landmarks;
            if (!lm) continue;

            // Face Contour -> 1 (Face Skin)
            if (lm.faceContour && lm.faceContour.pointCount >= 3) {
                rasterize_feature_region(out_parts_mask, mask_w, mask_h, lm.faceContour.normalizedPoints, lm.faceContour.pointCount, bb, 1);
            }
            // Eyebrows -> 3
            if (lm.leftEyebrow && lm.leftEyebrow.pointCount >= 3) {
                rasterize_feature_region(out_parts_mask, mask_w, mask_h, lm.leftEyebrow.normalizedPoints, lm.leftEyebrow.pointCount, bb, 3);
            }
            if (lm.rightEyebrow && lm.rightEyebrow.pointCount >= 3) {
                rasterize_feature_region(out_parts_mask, mask_w, mask_h, lm.rightEyebrow.normalizedPoints, lm.rightEyebrow.pointCount, bb, 3);
            }
            // Eyes -> 2 (Sclera + Iris/Pupil)
            if (lm.leftEye && lm.leftEye.pointCount >= 3) {
                rasterize_feature_region(out_parts_mask, mask_w, mask_h, lm.leftEye.normalizedPoints, lm.leftEye.pointCount, bb, 2);
            }
            if (lm.rightEye && lm.rightEye.pointCount >= 3) {
                rasterize_feature_region(out_parts_mask, mask_w, mask_h, lm.rightEye.normalizedPoints, lm.rightEye.pointCount, bb, 2);
            }
            // Outer Lips -> 4
            if (lm.outerLips && lm.outerLips.pointCount >= 3) {
                rasterize_feature_region(out_parts_mask, mask_w, mask_h, lm.outerLips.normalizedPoints, lm.outerLips.pointCount, bb, 4);
            }
            // Inner Lips (Teeth) -> 5
            if (lm.innerLips && lm.innerLips.pointCount >= 3) {
                rasterize_feature_region(out_parts_mask, mask_w, mask_h, lm.innerLips.normalizedPoints, lm.innerLips.pointCount, bb, 5);
            }
        }

        return 1;
    }
}

int lightraw_ai_extract_depth_rgba(
    const char *file_path_utf8,
    const uint8_t *rgba_ptr,
    uint32_t width,
    uint32_t height,
    uint8_t *out_depth,
    uint32_t depth_w,
    uint32_t depth_h
) {
    if (!out_depth || depth_w == 0 || depth_h == 0) return 0;
    @autoreleasepool {
        // 1. Try reading embedded hardware disparity / depth map from original photo file
        if (file_path_utf8 && strlen(file_path_utf8) > 0) {
            NSString *nsPath = [NSString stringWithUTF8String:file_path_utf8];
            NSURL *url = [NSURL fileURLWithPath:nsPath];
            CGImageSourceRef src = CGImageSourceCreateWithURL((__bridge CFURLRef)url, NULL);
            if (src) {
                CFDictionaryRef auxDisparity = CGImageSourceCopyAuxiliaryDataInfoAtIndex(src, 0, kCGImageAuxiliaryDataTypeDisparity);
                CFDictionaryRef auxDepth = auxDisparity ? auxDisparity : CGImageSourceCopyAuxiliaryDataInfoAtIndex(src, 0, kCGImageAuxiliaryDataTypeDepth);
                if (auxDepth) {
                    CFDataRef dData = (CFDataRef)CFDictionaryGetValue(auxDepth, kCGImageAuxiliaryDataInfoData);
                    CFDictionaryRef dDesc = (CFDictionaryRef)CFDictionaryGetValue(auxDepth, kCGImageAuxiliaryDataInfoDataDescription);
                    if (dData && dDesc) {
                        NSNumber *wNum = (NSNumber *)CFDictionaryGetValue(dDesc, kCGImagePropertyPixelWidth);
                        NSNumber *hNum = (NSNumber *)CFDictionaryGetValue(dDesc, kCGImagePropertyPixelHeight);
                        NSNumber *bprNum = (NSNumber *)CFDictionaryGetValue(dDesc, kCGImagePropertyBytesPerRow);
                        size_t dw = wNum ? wNum.unsignedIntegerValue : 0;
                        size_t dh = hNum ? hNum.unsignedIntegerValue : 0;
                        size_t dbpr = bprNum ? bprNum.unsignedIntegerValue : 0;
                        const uint8_t *raw_bytes = CFDataGetBytePtr(dData);
                        if (dw > 0 && dh > 0 && raw_bytes) {
                            for (uint32_t y = 0; y < depth_h; y++) {
                                size_t sy = (size_t)(((float)y / (float)depth_h) * (float)(dh - 1));
                                const uint8_t *row = raw_bytes + (dbpr > 0 ? sy * dbpr : sy * dw);
                                for (uint32_t x = 0; x < depth_w; x++) {
                                    size_t sx = (size_t)(((float)x / (float)depth_w) * (float)(dw - 1));
                                    out_depth[y * depth_w + x] = row[sx];
                                }
                            }
                            CFRelease(auxDepth);
                            CFRelease(src);
                            return 1;
                        }
                    }
                    CFRelease(auxDepth);
                }
                CFRelease(src);
            }
        }
        return 0; // Fallback to synthetic depth
    }
}
