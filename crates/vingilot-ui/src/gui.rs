use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use std::sync::{Mutex, OnceLock};

use crate::start_page::StartPageModel;
use vingilot_core::bookmark_manager::BookmarkStore;
use vingilot_core::db::Database;
use vingilot_core::previews::{default_data_dir, default_db_path, default_previews_dir};
use vingilot_core::repositories::HistoryRepository;
use vingilot_core::search_engine_manager::SearchEngineManager;

type GCallback = unsafe extern "C" fn();

// Global pointers to the active widgets so C signal callbacks can interact with them safely
static ACTIVE_WINDOW: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static ACTIVE_WEBVIEW: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static ACTIVE_ENTRY: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static ACTIVE_POPOVER: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static ACTIVE_ZOOM_LABEL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static ACTIVE_LENS_BTN: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static ACTIVE_SIDEBAR_BOX: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static ACTIVE_PANED: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static ACTIVE_PINNED_BOX: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static ACTIVE_SEPARATOR: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static ACTIVE_NORMAL_BOX: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static ACTIVE_RIGHT_SIDEBAR: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static ACTIVE_AI_BASE_URL_ENTRY: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static ACTIVE_AI_MODEL_ENTRY: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static ACTIVE_AI_API_KEY_ENTRY: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static ACTIVE_AI_STATUS_LABEL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static SIDEBAR_COLLAPSED: AtomicBool = AtomicBool::new(false);
const MIN_COLLAPSED_SIDEBAR_WIDTH: i32 = 62;
static TAB_MANAGER: Mutex<Option<crate::tabs::TabManager>> = Mutex::new(None);

// Start Page and Database statics
static BROWSER_DB: Mutex<Option<Database>> = Mutex::new(None);
static ACTIVE_PROFILE_ID: Mutex<Option<uuid::Uuid>> = Mutex::new(None);
static START_PAGE_MODEL: Mutex<Option<StartPageModel>> = Mutex::new(None);
static ACTIVE_CONTENT_STACK: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static ACTIVE_START_PAGE_SEARCH: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static ACTIVE_RECENT_BOX: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static ACTIVE_RECENT_FLOW: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static ACTIVE_BOOKMARKS_FLOW: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static ACTIVE_BREADCRUMBS_BOX: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static DRAGGED_ITEM_ID: Mutex<Option<uuid::Uuid>> = Mutex::new(None);
static VISIBLE_FOLDER_TILES: Mutex<Vec<(uuid::Uuid, usize)>> = Mutex::new(Vec::new());
static VISIBLE_BREADCRUMB_BTNS: Mutex<Vec<(Option<uuid::Uuid>, usize)>> = Mutex::new(Vec::new());
static ACTIVE_RIGHT_PANED: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static ACTIVE_STAR_BTN: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static ACCENT_CSS_PROVIDER: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

#[derive(Clone, Copy)]
#[allow(dead_code)]
struct GtkBindings {
    // Adwaita
    adw_init: unsafe extern "C" fn(),
    adw_application_new: unsafe extern "C" fn(*const c_char, c_int) -> *mut c_void,
    adw_application_window_new: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
    adw_application_window_set_content: unsafe extern "C" fn(*mut c_void, *mut c_void),
    adw_header_bar_new: unsafe extern "C" fn() -> *mut c_void,
    adw_header_bar_pack_start: unsafe extern "C" fn(*mut c_void, *mut c_void),
    adw_header_bar_pack_end: unsafe extern "C" fn(*mut c_void, *mut c_void),
    adw_header_bar_set_title_widget: unsafe extern "C" fn(*mut c_void, *mut c_void),
    adw_header_bar_set_show_start_title_buttons: unsafe extern "C" fn(*mut c_void, c_int),

    // Adwaita Dialogs & Preferences
    adw_about_dialog_new: unsafe extern "C" fn() -> *mut c_void,
    adw_about_dialog_set_application_name: unsafe extern "C" fn(*mut c_void, *const c_char),
    adw_about_dialog_set_application_icon: unsafe extern "C" fn(*mut c_void, *const c_char),
    adw_about_dialog_set_version: unsafe extern "C" fn(*mut c_void, *const c_char),
    adw_about_dialog_set_developer_name: unsafe extern "C" fn(*mut c_void, *const c_char),
    adw_about_dialog_set_comments: unsafe extern "C" fn(*mut c_void, *const c_char),
    adw_about_dialog_set_website: unsafe extern "C" fn(*mut c_void, *const c_char),
    adw_about_dialog_set_issue_url: unsafe extern "C" fn(*mut c_void, *const c_char),
    adw_dialog_present: unsafe extern "C" fn(*mut c_void, *mut c_void),

    adw_preferences_window_new: unsafe extern "C" fn() -> *mut c_void,
    adw_preferences_window_add: unsafe extern "C" fn(*mut c_void, *mut c_void),
    adw_preferences_page_new: unsafe extern "C" fn() -> *mut c_void,
    adw_preferences_page_set_title: unsafe extern "C" fn(*mut c_void, *const c_char),
    adw_preferences_page_set_icon_name: unsafe extern "C" fn(*mut c_void, *const c_char),
    adw_preferences_page_add: unsafe extern "C" fn(*mut c_void, *mut c_void),
    adw_preferences_group_new: unsafe extern "C" fn() -> *mut c_void,
    adw_preferences_group_set_title: unsafe extern "C" fn(*mut c_void, *const c_char),
    adw_preferences_group_add: unsafe extern "C" fn(*mut c_void, *mut c_void),
    adw_action_row_new: unsafe extern "C" fn() -> *mut c_void,
    adw_preferences_row_set_title: unsafe extern "C" fn(*mut c_void, *const c_char),
    adw_action_row_set_subtitle: unsafe extern "C" fn(*mut c_void, *const c_char),
    adw_switch_row_new: unsafe extern "C" fn() -> *mut c_void,
    adw_switch_row_set_active: unsafe extern "C" fn(*mut c_void, c_int),

    // GTK4
    gtk_window_set_title: unsafe extern "C" fn(*mut c_void, *const c_char),
    gtk_window_set_default_size: unsafe extern "C" fn(*mut c_void, c_int, c_int),
    gtk_window_set_transient_for: unsafe extern "C" fn(*mut c_void, *mut c_void),
    gtk_window_set_modal: unsafe extern "C" fn(*mut c_void, c_int),
    gtk_window_present: unsafe extern "C" fn(*mut c_void),
    gtk_box_new: unsafe extern "C" fn(c_int, c_int) -> *mut c_void,
    gtk_box_append: unsafe extern "C" fn(*mut c_void, *mut c_void),
    gtk_box_remove: unsafe extern "C" fn(*mut c_void, *mut c_void),
    gtk_widget_set_vexpand: unsafe extern "C" fn(*mut c_void, c_int),
    gtk_widget_set_hexpand: unsafe extern "C" fn(*mut c_void, c_int),
    gtk_widget_set_size_request: unsafe extern "C" fn(*mut c_void, c_int, c_int),
    gtk_widget_set_visible: unsafe extern "C" fn(*mut c_void, c_int),
    gtk_widget_get_visible: unsafe extern "C" fn(*mut c_void) -> c_int,
    gtk_widget_get_first_child: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
    gtk_widget_add_css_class: unsafe extern "C" fn(*mut c_void, *const c_char),
    gtk_paned_new: unsafe extern "C" fn(c_int) -> *mut c_void,
    gtk_paned_set_start_child: unsafe extern "C" fn(*mut c_void, *mut c_void),
    gtk_paned_set_end_child: unsafe extern "C" fn(*mut c_void, *mut c_void),
    gtk_paned_set_resize_start_child: unsafe extern "C" fn(*mut c_void, c_int),
    gtk_paned_set_shrink_start_child: unsafe extern "C" fn(*mut c_void, c_int),
    gtk_paned_set_resize_end_child: unsafe extern "C" fn(*mut c_void, c_int),
    gtk_paned_set_shrink_end_child: unsafe extern "C" fn(*mut c_void, c_int),
    gtk_paned_set_position: unsafe extern "C" fn(*mut c_void, c_int),
    gtk_paned_get_position: unsafe extern "C" fn(*mut c_void) -> c_int,
    gtk_paned_set_wide_handle: unsafe extern "C" fn(*mut c_void, c_int),
    gdk_display_get_default: unsafe extern "C" fn() -> *mut c_void,
    gtk_icon_theme_get_for_display: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
    gtk_icon_theme_add_search_path: unsafe extern "C" fn(*mut c_void, *const c_char),
    gtk_scrolled_window_new: unsafe extern "C" fn() -> *mut c_void,
    gtk_scrolled_window_set_child: unsafe extern "C" fn(*mut c_void, *mut c_void),
    gtk_scrolled_window_set_policy: unsafe extern "C" fn(*mut c_void, c_int, c_int),
    gtk_button_new: unsafe extern "C" fn() -> *mut c_void,
    gtk_button_new_from_icon_name: unsafe extern "C" fn(*const c_char) -> *mut c_void,
    gtk_button_new_with_label: unsafe extern "C" fn(*const c_char) -> *mut c_void,
    gtk_button_set_child: unsafe extern "C" fn(*mut c_void, *mut c_void),
    gtk_button_set_has_frame: unsafe extern "C" fn(*mut c_void, c_int),
    gtk_menu_button_new: unsafe extern "C" fn() -> *mut c_void,
    gtk_menu_button_set_icon_name: unsafe extern "C" fn(*mut c_void, *const c_char),
    gtk_menu_button_set_label: unsafe extern "C" fn(*mut c_void, *const c_char),
    gtk_menu_button_set_has_frame: unsafe extern "C" fn(*mut c_void, c_int),
    gtk_menu_button_set_popover: unsafe extern "C" fn(*mut c_void, *mut c_void),
    gtk_popover_new: unsafe extern "C" fn() -> *mut c_void,
    gtk_popover_set_child: unsafe extern "C" fn(*mut c_void, *mut c_void),
    gtk_popover_popdown: unsafe extern "C" fn(*mut c_void),
    gtk_label_new: unsafe extern "C" fn(*const c_char) -> *mut c_void,
    gtk_label_set_text: unsafe extern "C" fn(*mut c_void, *const c_char),
    gtk_label_set_xalign: unsafe extern "C" fn(*mut c_void, libc::c_float),
    gtk_label_set_ellipsize: unsafe extern "C" fn(*mut c_void, c_int),
    gtk_separator_new: unsafe extern "C" fn(c_int) -> *mut c_void,
    gtk_switch_new: unsafe extern "C" fn() -> *mut c_void,
    gtk_switch_set_active: unsafe extern "C" fn(*mut c_void, c_int),
    gtk_switch_get_active: unsafe extern "C" fn(*mut c_void) -> c_int,
    gtk_widget_set_margin_start: unsafe extern "C" fn(*mut c_void, c_int),
    gtk_widget_set_margin_end: unsafe extern "C" fn(*mut c_void, c_int),
    gtk_widget_set_margin_top: unsafe extern "C" fn(*mut c_void, c_int),
    gtk_widget_set_margin_bottom: unsafe extern "C" fn(*mut c_void, c_int),
    gtk_widget_set_tooltip_text: unsafe extern "C" fn(*mut c_void, *const c_char),
    gtk_entry_new: unsafe extern "C" fn() -> *mut c_void,
    gtk_entry_set_placeholder_text: unsafe extern "C" fn(*mut c_void, *const c_char),
    gtk_editable_get_text: unsafe extern "C" fn(*mut c_void) -> *const c_char,
    gtk_editable_set_text: unsafe extern "C" fn(*mut c_void, *const c_char),
    gtk_image_new_from_icon_name: unsafe extern "C" fn(*const c_char) -> *mut c_void,
    gtk_image_new_from_paintable: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
    gtk_image_set_from_paintable: unsafe extern "C" fn(*mut c_void, *mut c_void),
    gtk_image_set_pixel_size: unsafe extern "C" fn(*mut c_void, c_int),
    gtk_gesture_drag_new: unsafe extern "C" fn() -> *mut c_void,
    gtk_gesture_drag_get_start_point:
        unsafe extern "C" fn(*mut c_void, *mut libc::c_double, *mut libc::c_double) -> c_int,
    gtk_widget_add_controller: unsafe extern "C" fn(*mut c_void, *mut c_void),
    gtk_widget_translate_coordinates: unsafe extern "C" fn(
        *mut c_void,
        *mut c_void,
        libc::c_double,
        libc::c_double,
        *mut libc::c_double,
        *mut libc::c_double,
    ) -> c_int,
    gtk_event_controller_get_widget: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
    gtk_stack_new: unsafe extern "C" fn() -> *mut c_void,
    gtk_stack_add_named: unsafe extern "C" fn(*mut c_void, *mut c_void, *const c_char),
    gtk_stack_set_visible_child_name: unsafe extern "C" fn(*mut c_void, *const c_char),
    gtk_stack_get_visible_child_name: unsafe extern "C" fn(*mut c_void) -> *const c_char,
    gtk_flow_box_new: unsafe extern "C" fn() -> *mut c_void,
    gtk_flow_box_append: unsafe extern "C" fn(*mut c_void, *mut c_void),
    gtk_flow_box_remove: unsafe extern "C" fn(*mut c_void, *mut c_void),
    gtk_flow_box_set_min_children_per_line: unsafe extern "C" fn(*mut c_void, libc::c_uint),
    gtk_flow_box_set_max_children_per_line: unsafe extern "C" fn(*mut c_void, libc::c_uint),
    gtk_flow_box_set_selection_mode: unsafe extern "C" fn(*mut c_void, c_int),
    gtk_flow_box_set_column_spacing: unsafe extern "C" fn(*mut c_void, libc::c_uint),
    gtk_flow_box_set_row_spacing: unsafe extern "C" fn(*mut c_void, libc::c_uint),
    gtk_flow_box_set_homogeneous: unsafe extern "C" fn(*mut c_void, c_int),
    gtk_picture_new_for_filename: unsafe extern "C" fn(*const c_char) -> *mut c_void,
    gtk_widget_set_halign: unsafe extern "C" fn(*mut c_void, c_int),
    gtk_widget_set_valign: unsafe extern "C" fn(*mut c_void, c_int),
    gtk_widget_get_width: unsafe extern "C" fn(*mut c_void) -> c_int,
    gtk_widget_get_height: unsafe extern "C" fn(*mut c_void) -> c_int,
    gtk_gesture_click_new: unsafe extern "C" fn() -> *mut c_void,
    gtk_gesture_single_set_button: unsafe extern "C" fn(*mut c_void, libc::c_uint),
    gtk_css_provider_new: unsafe extern "C" fn() -> *mut c_void,
    gtk_css_provider_load_from_string: unsafe extern "C" fn(*mut c_void, *const c_char),
    gtk_style_context_add_provider_for_display:
        unsafe extern "C" fn(*mut c_void, *mut c_void, libc::c_uint),

    // WebKitGTK
    webkit_web_view_new: unsafe extern "C" fn() -> *mut c_void,
    webkit_web_view_load_uri: unsafe extern "C" fn(*mut c_void, *const c_char),
    webkit_web_view_get_uri: unsafe extern "C" fn(*mut c_void) -> *const c_char,
    webkit_web_view_get_title: unsafe extern "C" fn(*mut c_void) -> *const c_char,
    webkit_web_view_get_favicon: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
    webkit_web_view_go_back: unsafe extern "C" fn(*mut c_void),
    webkit_web_view_go_forward: unsafe extern "C" fn(*mut c_void),
    webkit_web_view_reload: unsafe extern "C" fn(*mut c_void),
    webkit_web_view_get_zoom_level: unsafe extern "C" fn(*mut c_void) -> libc::c_double,
    webkit_web_view_set_zoom_level: unsafe extern "C" fn(*mut c_void, libc::c_double),
    webkit_web_view_evaluate_javascript: unsafe extern "C" fn(
        *mut c_void,
        *const c_char,
        libc::ssize_t,
        *const c_char,
        *const c_char,
        *mut c_void,
        *mut c_void,
        *mut c_void,
    ),

    // GObject / GIO
    g_signal_connect_data: unsafe extern "C" fn(
        *mut c_void,
        *const c_char,
        GCallback,
        *mut c_void,
        *mut c_void,
        c_int,
    ) -> libc::c_ulong,
    g_application_run: unsafe extern "C" fn(*mut c_void, c_int, *const *const c_char) -> c_int,
    g_object_unref: unsafe extern "C" fn(*mut c_void),

    // GLib
    g_idle_add: unsafe extern "C" fn(
        unsafe extern "C" fn(*mut c_void) -> libc::c_int,
        *mut c_void,
    ) -> libc::c_uint,
}

unsafe impl Send for GtkBindings {}
unsafe impl Sync for GtkBindings {}

static BINDINGS: OnceLock<GtkBindings> = OnceLock::new();
static IS_INCOGNITO: AtomicBool = AtomicBool::new(false);
static INITIAL_URL: Mutex<Option<String>> = Mutex::new(None);

unsafe fn load_symbol<T>(handle: *mut c_void, symbol_name: &str) -> Option<T> {
    let c_name = CString::new(symbol_name).ok()?;
    let symbol = libc::dlsym(handle, c_name.as_ptr());
    if symbol.is_null() {
        None
    } else {
        Some(std::mem::transmute_copy(&symbol))
    }
}

unsafe fn load_bindings() -> Option<GtkBindings> {
    let flags = libc::RTLD_NOW | libc::RTLD_GLOBAL;

    let libgtk_name = CString::new("libgtk-4.so.1").ok()?;
    let libgtk = libc::dlopen(libgtk_name.as_ptr(), flags);
    if libgtk.is_null() {
        return None;
    }

    let libadw_name = CString::new("libadwaita-1.so.0").ok()?;
    let libadw = libc::dlopen(libadw_name.as_ptr(), flags);
    if libadw.is_null() {
        return None;
    }

    let libwebkit_name = CString::new("libwebkitgtk-6.0.so.4").ok()?;
    let libwebkit = libc::dlopen(libwebkit_name.as_ptr(), flags);
    if libwebkit.is_null() {
        return None;
    }

    let libgobj_name = CString::new("libgobject-2.0.so.0").ok()?;
    let libgobj = libc::dlopen(libgobj_name.as_ptr(), flags);
    if libgobj.is_null() {
        return None;
    }

    let libgio_name = CString::new("libgio-2.0.so.0").ok()?;
    let libgio = libc::dlopen(libgio_name.as_ptr(), flags);
    if libgio.is_null() {
        return None;
    }

    let libglib_name = CString::new("libglib-2.0.so.0").ok()?;
    let libglib = libc::dlopen(libglib_name.as_ptr(), flags);
    if libglib.is_null() {
        return None;
    }

    Some(GtkBindings {
        adw_init: load_symbol(libadw, "adw_init")?,
        adw_application_new: load_symbol(libadw, "adw_application_new")?,
        adw_application_window_new: load_symbol(libadw, "adw_application_window_new")?,
        adw_application_window_set_content: load_symbol(
            libadw,
            "adw_application_window_set_content",
        )?,
        adw_header_bar_new: load_symbol(libadw, "adw_header_bar_new")?,
        adw_header_bar_pack_start: load_symbol(libadw, "adw_header_bar_pack_start")?,
        adw_header_bar_pack_end: load_symbol(libadw, "adw_header_bar_pack_end")?,
        adw_header_bar_set_title_widget: load_symbol(libadw, "adw_header_bar_set_title_widget")?,
        adw_header_bar_set_show_start_title_buttons: load_symbol(
            libadw,
            "adw_header_bar_set_show_start_title_buttons",
        )?,

        adw_about_dialog_new: load_symbol(libadw, "adw_about_dialog_new")?,
        adw_about_dialog_set_application_name: load_symbol(
            libadw,
            "adw_about_dialog_set_application_name",
        )?,
        adw_about_dialog_set_application_icon: load_symbol(
            libadw,
            "adw_about_dialog_set_application_icon",
        )?,
        adw_about_dialog_set_version: load_symbol(libadw, "adw_about_dialog_set_version")?,
        adw_about_dialog_set_developer_name: load_symbol(
            libadw,
            "adw_about_dialog_set_developer_name",
        )?,
        adw_about_dialog_set_comments: load_symbol(libadw, "adw_about_dialog_set_comments")?,
        adw_about_dialog_set_website: load_symbol(libadw, "adw_about_dialog_set_website")?,
        adw_about_dialog_set_issue_url: load_symbol(libadw, "adw_about_dialog_set_issue_url")?,
        adw_dialog_present: load_symbol(libadw, "adw_dialog_present")?,

        adw_preferences_window_new: load_symbol(libadw, "adw_preferences_window_new")?,
        adw_preferences_window_add: load_symbol(libadw, "adw_preferences_window_add")?,
        adw_preferences_page_new: load_symbol(libadw, "adw_preferences_page_new")?,
        adw_preferences_page_set_title: load_symbol(libadw, "adw_preferences_page_set_title")?,
        adw_preferences_page_set_icon_name: load_symbol(
            libadw,
            "adw_preferences_page_set_icon_name",
        )?,
        adw_preferences_page_add: load_symbol(libadw, "adw_preferences_page_add")?,
        adw_preferences_group_new: load_symbol(libadw, "adw_preferences_group_new")?,
        adw_preferences_group_set_title: load_symbol(libadw, "adw_preferences_group_set_title")?,
        adw_preferences_group_add: load_symbol(libadw, "adw_preferences_group_add")?,
        adw_action_row_new: load_symbol(libadw, "adw_action_row_new")?,
        adw_preferences_row_set_title: load_symbol(libadw, "adw_preferences_row_set_title")?,
        adw_action_row_set_subtitle: load_symbol(libadw, "adw_action_row_set_subtitle")?,
        adw_switch_row_new: load_symbol(libadw, "adw_switch_row_new")?,
        adw_switch_row_set_active: load_symbol(libadw, "adw_switch_row_set_active")?,

        gtk_window_set_title: load_symbol(libgtk, "gtk_window_set_title")?,
        gtk_window_set_default_size: load_symbol(libgtk, "gtk_window_set_default_size")?,
        gtk_window_set_transient_for: load_symbol(libgtk, "gtk_window_set_transient_for")?,
        gtk_window_set_modal: load_symbol(libgtk, "gtk_window_set_modal")?,
        gtk_window_present: load_symbol(libgtk, "gtk_window_present")?,
        gtk_box_new: load_symbol(libgtk, "gtk_box_new")?,
        gtk_box_append: load_symbol(libgtk, "gtk_box_append")?,
        gtk_box_remove: load_symbol(libgtk, "gtk_box_remove")?,
        gtk_widget_set_vexpand: load_symbol(libgtk, "gtk_widget_set_vexpand")?,
        gtk_widget_set_hexpand: load_symbol(libgtk, "gtk_widget_set_hexpand")?,
        gtk_widget_set_size_request: load_symbol(libgtk, "gtk_widget_set_size_request")?,
        gtk_widget_set_visible: load_symbol(libgtk, "gtk_widget_set_visible")?,
        gtk_widget_get_visible: load_symbol(libgtk, "gtk_widget_get_visible")?,
        gtk_widget_get_first_child: load_symbol(libgtk, "gtk_widget_get_first_child")?,
        gtk_widget_add_css_class: load_symbol(libgtk, "gtk_widget_add_css_class")?,
        gtk_paned_new: load_symbol(libgtk, "gtk_paned_new")?,
        gtk_paned_set_start_child: load_symbol(libgtk, "gtk_paned_set_start_child")?,
        gtk_paned_set_end_child: load_symbol(libgtk, "gtk_paned_set_end_child")?,
        gtk_paned_set_resize_start_child: load_symbol(libgtk, "gtk_paned_set_resize_start_child")?,
        gtk_paned_set_shrink_start_child: load_symbol(libgtk, "gtk_paned_set_shrink_start_child")?,
        gtk_paned_set_resize_end_child: load_symbol(libgtk, "gtk_paned_set_resize_end_child")?,
        gtk_paned_set_shrink_end_child: load_symbol(libgtk, "gtk_paned_set_shrink_end_child")?,
        gtk_paned_set_position: load_symbol(libgtk, "gtk_paned_set_position")?,
        gtk_paned_get_position: load_symbol(libgtk, "gtk_paned_get_position")?,
        gtk_paned_set_wide_handle: load_symbol(libgtk, "gtk_paned_set_wide_handle")?,
        gdk_display_get_default: load_symbol(libgtk, "gdk_display_get_default")?,
        gtk_icon_theme_get_for_display: load_symbol(libgtk, "gtk_icon_theme_get_for_display")?,
        gtk_icon_theme_add_search_path: load_symbol(libgtk, "gtk_icon_theme_add_search_path")?,
        gtk_scrolled_window_new: load_symbol(libgtk, "gtk_scrolled_window_new")?,
        gtk_scrolled_window_set_child: load_symbol(libgtk, "gtk_scrolled_window_set_child")?,
        gtk_scrolled_window_set_policy: load_symbol(libgtk, "gtk_scrolled_window_set_policy")?,
        gtk_button_new: load_symbol(libgtk, "gtk_button_new")?,
        gtk_button_new_from_icon_name: load_symbol(libgtk, "gtk_button_new_from_icon_name")?,
        gtk_button_new_with_label: load_symbol(libgtk, "gtk_button_new_with_label")?,
        gtk_button_set_child: load_symbol(libgtk, "gtk_button_set_child")?,
        gtk_button_set_has_frame: load_symbol(libgtk, "gtk_button_set_has_frame")?,
        gtk_menu_button_new: load_symbol(libgtk, "gtk_menu_button_new")?,
        gtk_menu_button_set_icon_name: load_symbol(libgtk, "gtk_menu_button_set_icon_name")?,
        gtk_menu_button_set_label: load_symbol(libgtk, "gtk_menu_button_set_label")?,
        gtk_menu_button_set_has_frame: load_symbol(libgtk, "gtk_menu_button_set_has_frame")?,
        gtk_menu_button_set_popover: load_symbol(libgtk, "gtk_menu_button_set_popover")?,
        gtk_popover_new: load_symbol(libgtk, "gtk_popover_new")?,
        gtk_popover_set_child: load_symbol(libgtk, "gtk_popover_set_child")?,
        gtk_popover_popdown: load_symbol(libgtk, "gtk_popover_popdown")?,
        gtk_label_new: load_symbol(libgtk, "gtk_label_new")?,
        gtk_label_set_text: load_symbol(libgtk, "gtk_label_set_text")?,
        gtk_label_set_xalign: load_symbol(libgtk, "gtk_label_set_xalign")?,
        gtk_label_set_ellipsize: load_symbol(libgtk, "gtk_label_set_ellipsize")?,
        gtk_separator_new: load_symbol(libgtk, "gtk_separator_new")?,
        gtk_switch_new: load_symbol(libgtk, "gtk_switch_new")?,
        gtk_switch_set_active: load_symbol(libgtk, "gtk_switch_set_active")?,
        gtk_switch_get_active: load_symbol(libgtk, "gtk_switch_get_active")?,
        gtk_widget_set_margin_start: load_symbol(libgtk, "gtk_widget_set_margin_start")?,
        gtk_widget_set_margin_end: load_symbol(libgtk, "gtk_widget_set_margin_end")?,
        gtk_widget_set_margin_top: load_symbol(libgtk, "gtk_widget_set_margin_top")?,
        gtk_widget_set_margin_bottom: load_symbol(libgtk, "gtk_widget_set_margin_bottom")?,
        gtk_widget_set_tooltip_text: load_symbol(libgtk, "gtk_widget_set_tooltip_text")?,
        gtk_entry_new: load_symbol(libgtk, "gtk_entry_new")?,
        gtk_entry_set_placeholder_text: load_symbol(libgtk, "gtk_entry_set_placeholder_text")?,
        gtk_editable_get_text: load_symbol(libgtk, "gtk_editable_get_text")?,
        gtk_editable_set_text: load_symbol(libgtk, "gtk_editable_set_text")?,
        gtk_image_new_from_icon_name: load_symbol(libgtk, "gtk_image_new_from_icon_name")?,
        gtk_image_new_from_paintable: load_symbol(libgtk, "gtk_image_new_from_paintable")?,
        gtk_image_set_from_paintable: load_symbol(libgtk, "gtk_image_set_from_paintable")?,
        gtk_image_set_pixel_size: load_symbol(libgtk, "gtk_image_set_pixel_size")?,
        gtk_gesture_drag_new: load_symbol(libgtk, "gtk_gesture_drag_new")?,
        gtk_gesture_drag_get_start_point: load_symbol(libgtk, "gtk_gesture_drag_get_start_point")?,
        gtk_widget_add_controller: load_symbol(libgtk, "gtk_widget_add_controller")?,
        gtk_widget_translate_coordinates: load_symbol(libgtk, "gtk_widget_translate_coordinates")?,
        gtk_event_controller_get_widget: load_symbol(libgtk, "gtk_event_controller_get_widget")?,
        gtk_stack_new: load_symbol(libgtk, "gtk_stack_new")?,
        gtk_stack_add_named: load_symbol(libgtk, "gtk_stack_add_named")?,
        gtk_stack_set_visible_child_name: load_symbol(libgtk, "gtk_stack_set_visible_child_name")?,
        gtk_stack_get_visible_child_name: load_symbol(libgtk, "gtk_stack_get_visible_child_name")?,
        gtk_flow_box_new: load_symbol(libgtk, "gtk_flow_box_new")?,
        gtk_flow_box_append: load_symbol(libgtk, "gtk_flow_box_append")?,
        gtk_flow_box_remove: load_symbol(libgtk, "gtk_flow_box_remove")?,
        gtk_flow_box_set_min_children_per_line: load_symbol(
            libgtk,
            "gtk_flow_box_set_min_children_per_line",
        )?,
        gtk_flow_box_set_max_children_per_line: load_symbol(
            libgtk,
            "gtk_flow_box_set_max_children_per_line",
        )?,
        gtk_flow_box_set_selection_mode: load_symbol(libgtk, "gtk_flow_box_set_selection_mode")?,
        gtk_flow_box_set_column_spacing: load_symbol(libgtk, "gtk_flow_box_set_column_spacing")?,
        gtk_flow_box_set_row_spacing: load_symbol(libgtk, "gtk_flow_box_set_row_spacing")?,
        gtk_flow_box_set_homogeneous: load_symbol(libgtk, "gtk_flow_box_set_homogeneous")?,
        gtk_picture_new_for_filename: load_symbol(libgtk, "gtk_picture_new_for_filename")?,
        gtk_widget_set_halign: load_symbol(libgtk, "gtk_widget_set_halign")?,
        gtk_widget_set_valign: load_symbol(libgtk, "gtk_widget_set_valign")?,
        gtk_widget_get_width: load_symbol(libgtk, "gtk_widget_get_width")?,
        gtk_widget_get_height: load_symbol(libgtk, "gtk_widget_get_height")?,
        gtk_gesture_click_new: load_symbol(libgtk, "gtk_gesture_click_new")?,
        gtk_gesture_single_set_button: load_symbol(libgtk, "gtk_gesture_single_set_button")?,
        gtk_css_provider_new: load_symbol(libgtk, "gtk_css_provider_new")?,
        gtk_css_provider_load_from_string: load_symbol(
            libgtk,
            "gtk_css_provider_load_from_string",
        )?,
        gtk_style_context_add_provider_for_display: load_symbol(
            libgtk,
            "gtk_style_context_add_provider_for_display",
        )?,

        webkit_web_view_new: load_symbol(libwebkit, "webkit_web_view_new")?,
        webkit_web_view_load_uri: load_symbol(libwebkit, "webkit_web_view_load_uri")?,
        webkit_web_view_get_uri: load_symbol(libwebkit, "webkit_web_view_get_uri")?,
        webkit_web_view_get_title: load_symbol(libwebkit, "webkit_web_view_get_title")?,
        webkit_web_view_get_favicon: load_symbol(libwebkit, "webkit_web_view_get_favicon")?,
        webkit_web_view_go_back: load_symbol(libwebkit, "webkit_web_view_go_back")?,
        webkit_web_view_go_forward: load_symbol(libwebkit, "webkit_web_view_go_forward")?,
        webkit_web_view_reload: load_symbol(libwebkit, "webkit_web_view_reload")?,
        webkit_web_view_get_zoom_level: load_symbol(libwebkit, "webkit_web_view_get_zoom_level")?,
        webkit_web_view_set_zoom_level: load_symbol(libwebkit, "webkit_web_view_set_zoom_level")?,
        webkit_web_view_evaluate_javascript: load_symbol(
            libwebkit,
            "webkit_web_view_evaluate_javascript",
        )?,

        g_signal_connect_data: load_symbol(libgobj, "g_signal_connect_data")?,
        g_application_run: load_symbol(libgio, "g_application_run")?,
        g_object_unref: load_symbol(libgobj, "g_object_unref")?,
        g_idle_add: load_symbol(libglib, "g_idle_add")?,
    })
}

unsafe extern "C" fn on_back_clicked(_button: *mut c_void, _user_data: *mut c_void) {
    let webview = ACTIVE_WEBVIEW.load(Ordering::SeqCst);
    if !webview.is_null() {
        if let Some(b) = BINDINGS.get() {
            (b.webkit_web_view_go_back)(webview);
        }
    }
}

unsafe extern "C" fn on_forward_clicked(_button: *mut c_void, _user_data: *mut c_void) {
    let webview = ACTIVE_WEBVIEW.load(Ordering::SeqCst);
    if !webview.is_null() {
        if let Some(b) = BINDINGS.get() {
            (b.webkit_web_view_go_forward)(webview);
        }
    }
}

unsafe extern "C" fn on_refresh_clicked(_button: *mut c_void, _user_data: *mut c_void) {
    let webview = ACTIVE_WEBVIEW.load(Ordering::SeqCst);
    if !webview.is_null() {
        if let Some(b) = BINDINGS.get() {
            (b.webkit_web_view_reload)(webview);
        }
    }
}

unsafe extern "C" fn on_entry_activate(entry: *mut c_void, _user_data: *mut c_void) {
    if entry.is_null() {
        return;
    }
    if let Some(b) = BINDINGS.get() {
        let text_ptr = (b.gtk_editable_get_text)(entry);
        if !text_ptr.is_null() {
            let text = CStr::from_ptr(text_ptr)
                .to_string_lossy()
                .trim()
                .to_string();
            if text.is_empty() {
                return;
            }
            let target_url = {
                let model_guard = START_PAGE_MODEL.lock().ok();
                if let Some(Some(model)) = model_guard.as_deref() {
                    model
                        .resolve_search_input(&text)
                        .unwrap_or_else(|| format!("https://duckduckgo.com/?q={}", text))
                } else if text.starts_with("http://")
                    || text.starts_with("https://")
                    || text.starts_with("about:")
                    || text.starts_with("file://")
                {
                    text
                } else if text.contains('.') && !text.contains(' ') {
                    format!("https://{}", text)
                } else {
                    format!("https://duckduckgo.com/?q={}", text)
                }
            };

            navigate_to_url(&target_url);
        }
    }
}

unsafe extern "C" fn on_tab_drag_end(
    gesture: *mut c_void,
    offset_x: libc::c_double,
    offset_y: libc::c_double,
    user_data: *mut c_void,
) {
    let idx = user_data as usize;
    let dist_sq = offset_x * offset_x + offset_y * offset_y;
    if dist_sq < 144.0 {
        return;
    }
    let b = match BINDINGS.get() {
        Some(b) => b,
        None => return,
    };
    let sep = ACTIVE_SEPARATOR.load(Ordering::SeqCst);
    let widget = (b.gtk_event_controller_get_widget)(gesture);
    if widget.is_null() {
        return;
    }

    let mut start_x = 0.0;
    let mut start_y = 0.0;
    (b.gtk_gesture_drag_get_start_point)(gesture, &mut start_x, &mut start_y);
    let drop_x = start_x + offset_x;
    let drop_y = start_y + offset_y;

    let mut is_above_sep = None;

    if !sep.is_null() {
        let mut sep_rel_x = 0.0;
        let mut sep_rel_y = 0.0;
        let ok = (b.gtk_widget_translate_coordinates)(
            widget,
            sep,
            drop_x,
            drop_y,
            &mut sep_rel_x,
            &mut sep_rel_y,
        );
        if ok != 0 {
            is_above_sep = Some(sep_rel_y < 0.0);
        }
    }

    if let Ok(mut mgr_guard) = TAB_MANAGER.lock() {
        if let Some(mgr) = mgr_guard.as_mut() {
            let was_pinned = mgr.tabs().get(idx).map(|t| t.is_pinned).unwrap_or(false);
            if let Some(above) = is_above_sep {
                if above != was_pinned {
                    mgr.set_pinned(idx, above);
                } else if offset_y < -24.0 && idx > 0 {
                    mgr.reorder_tab(idx, idx - 1);
                } else if offset_y > 24.0 && idx + 1 < mgr.tabs().len() {
                    mgr.reorder_tab(idx, idx + 1);
                }
            } else {
                if offset_y < -35.0 && !was_pinned {
                    mgr.set_pinned(idx, true);
                } else if offset_y > 35.0 && was_pinned {
                    mgr.set_pinned(idx, false);
                } else if offset_y < -24.0 && idx > 0 {
                    mgr.reorder_tab(idx, idx - 1);
                } else if offset_y > 24.0 && idx + 1 < mgr.tabs().len() {
                    mgr.reorder_tab(idx, idx + 1);
                }
            }
        }
    }

    refresh_sidebar_tabs();
}

unsafe extern "C" fn on_tab_pin_toggle_clicked(_button: *mut c_void, user_data: *mut c_void) {
    let idx = user_data as usize;
    if let Ok(mut mgr_guard) = TAB_MANAGER.lock() {
        if let Some(mgr) = mgr_guard.as_mut() {
            mgr.toggle_pin(idx);
        }
    }
    refresh_sidebar_tabs();
}

unsafe extern "C" fn on_paned_position_notify(
    paned: *mut c_void,
    _pspec: *mut c_void,
    _user_data: *mut c_void,
) {
    let b = match BINDINGS.get() {
        Some(b) => b,
        None => return,
    };
    let pos = (b.gtk_paned_get_position)(paned);
    if pos < MIN_COLLAPSED_SIDEBAR_WIDTH {
        (b.gtk_paned_set_position)(paned, MIN_COLLAPSED_SIDEBAR_WIDTH);
    }
    let currently_collapsed = SIDEBAR_COLLAPSED.load(Ordering::SeqCst);

    if pos <= 75 && !currently_collapsed {
        SIDEBAR_COLLAPSED.store(true, Ordering::SeqCst);
        refresh_sidebar_tabs();
    } else if pos > 75 && currently_collapsed {
        SIDEBAR_COLLAPSED.store(false, Ordering::SeqCst);
        refresh_sidebar_tabs();
    }
}

unsafe extern "C" fn on_webview_favicon_notify(
    webview: *mut c_void,
    _pspec: *mut c_void,
    _user_data: *mut c_void,
) {
    let b = match BINDINGS.get() {
        Some(b) => b,
        None => return,
    };
    let favicon = (b.webkit_web_view_get_favicon)(webview);
    if !favicon.is_null() {
        if let Ok(mut mgr_guard) = TAB_MANAGER.lock() {
            if let Some(mgr) = mgr_guard.as_mut() {
                if let Some(active) = mgr.active_tab_mut() {
                    active.favicon_ptr = favicon as usize;
                }
            }
        }
        refresh_sidebar_tabs();
    }
}

unsafe extern "C" fn on_webview_title_notify(
    webview: *mut c_void,
    _pspec: *mut c_void,
    _user_data: *mut c_void,
) {
    let b = match BINDINGS.get() {
        Some(b) => b,
        None => return,
    };
    let title_ptr = (b.webkit_web_view_get_title)(webview);
    if !title_ptr.is_null() {
        let title_str = CStr::from_ptr(title_ptr).to_string_lossy().to_string();
        if !title_str.trim().is_empty() {
            let mut active_url = None;
            if let Ok(mut mgr_guard) = TAB_MANAGER.lock() {
                if let Some(mgr) = mgr_guard.as_mut() {
                    if let Some(active) = mgr.active_tab_mut() {
                        if !active.is_incognito {
                            active.title = title_str.clone();
                            active_url = Some(active.url.clone());
                        }
                    }
                }
            }
            let is_incog = IS_INCOGNITO.load(Ordering::SeqCst);
            if !is_incog {
                if let Some(url) = active_url {
                    if !url.starts_with("about:") && !url.starts_with("browser://") {
                        let db_guard = BROWSER_DB.lock().ok();
                        let prof_guard = ACTIVE_PROFILE_ID.lock().ok();
                        let db = db_guard.as_ref().and_then(|g| g.as_ref());
                        let prof_id = prof_guard.as_ref().and_then(|g| g.as_ref()).copied();
                        if let (Some(db), Some(profile_id)) = (db, prof_id) {
                            let hist_repo = HistoryRepository::new(db);
                            let _ = hist_repo.record_visit(profile_id, &url, Some(&title_str));
                        }
                    }
                }
            }
            refresh_sidebar_tabs();
        }
    }
}

unsafe fn update_star_status_for_uri(uri: &str) {
    let star_btn = ACTIVE_STAR_BTN.load(Ordering::SeqCst);
    if star_btn.is_null() {
        return;
    }
    let b = match BINDINGS.get() {
        Some(b) => b,
        None => return,
    };

    let is_bookmarked = {
        let db_guard = BROWSER_DB.lock().ok();
        if let Some(Some(db)) = db_guard.as_deref() {
            let store = BookmarkStore::new(db);
            store
                .get_root_items()
                .map(|items| items.iter().any(|item| item.url.as_deref() == Some(uri)))
                .unwrap_or(false)
        } else {
            false
        }
    };

    let icon_name = if is_bookmarked {
        "starred-symbolic"
    } else {
        "non-starred-symbolic"
    };
    let tip = if is_bookmarked {
        "Bookmarked (In your collection)"
    } else {
        "Add this page to Bookmarks"
    };
    let star_icon = (b.gtk_image_new_from_icon_name)(CString::new(icon_name).unwrap().as_ptr());
    (b.gtk_image_set_pixel_size)(star_icon, 18);
    (b.gtk_button_set_child)(star_btn, star_icon);
    (b.gtk_widget_set_tooltip_text)(star_btn, CString::new(tip).unwrap().as_ptr());
}

unsafe extern "C" fn on_star_clicked(_button: *mut c_void, _user_data: *mut c_void) {
    let b = match BINDINGS.get() {
        Some(b) => b,
        None => return,
    };
    let webview = ACTIVE_WEBVIEW.load(Ordering::SeqCst);
    let mut url_to_bookmark = String::new();
    let mut title_to_bookmark = String::new();

    if !webview.is_null() {
        let uri_ptr = (b.webkit_web_view_get_uri)(webview);
        if !uri_ptr.is_null() {
            url_to_bookmark = CStr::from_ptr(uri_ptr).to_string_lossy().trim().to_string();
        }
        let title_ptr = (b.webkit_web_view_get_title)(webview);
        if !title_ptr.is_null() {
            title_to_bookmark = CStr::from_ptr(title_ptr)
                .to_string_lossy()
                .trim()
                .to_string();
        }
    }

    if url_to_bookmark.is_empty()
        || url_to_bookmark == "about:blank"
        || url_to_bookmark == "browser://start"
    {
        let entry = ACTIVE_ENTRY.load(Ordering::SeqCst);
        if !entry.is_null() {
            let text_ptr = (b.gtk_editable_get_text)(entry);
            if !text_ptr.is_null() {
                url_to_bookmark = CStr::from_ptr(text_ptr)
                    .to_string_lossy()
                    .trim()
                    .to_string();
            }
        }
    }

    if url_to_bookmark.is_empty()
        || url_to_bookmark == "about:blank"
        || url_to_bookmark == "browser://start"
    {
        return;
    }

    if title_to_bookmark.is_empty() {
        title_to_bookmark = url_to_bookmark.clone();
    }

    {
        let db_guard = BROWSER_DB.lock().ok();
        let prof_guard = ACTIVE_PROFILE_ID.lock().ok();
        let mut model_guard = START_PAGE_MODEL.lock().ok();

        let db = db_guard.as_ref().and_then(|g| g.as_ref());
        let prof_id = prof_guard.as_ref().and_then(|g| g.as_ref()).copied();
        let model = model_guard.as_mut().and_then(|g| g.as_mut());

        if let (Some(db), Some(profile_id), Some(model)) = (db, prof_id, model) {
            let _ = model.create_bookmark(db, profile_id, &title_to_bookmark, &url_to_bookmark);
        }
    }

    let star_btn = ACTIVE_STAR_BTN.load(Ordering::SeqCst);
    if !star_btn.is_null() {
        let star_icon =
            (b.gtk_image_new_from_icon_name)(CString::new("starred-symbolic").unwrap().as_ptr());
        (b.gtk_image_set_pixel_size)(star_icon, 18);
        (b.gtk_button_set_child)(star_btn, star_icon);
        (b.gtk_widget_set_tooltip_text)(
            star_btn,
            CString::new("✓ Added to Bookmarks!").unwrap().as_ptr(),
        );
    }

    queue_refresh_start_page();
}

unsafe extern "C" fn on_reader_mode_clicked(_button: *mut c_void, _user_data: *mut c_void) {
    let webview = ACTIVE_WEBVIEW.load(Ordering::SeqCst);
    if webview.is_null() {
        return;
    }
    let b = match BINDINGS.get() {
        Some(b) => b,
        None => return,
    };

    let script = r#"
        (function() {
            let existing = document.getElementById('vingilot-reader-mode-style');
            if (existing) {
                existing.remove();
                return;
            }
            let style = document.createElement('style');
            style.id = 'vingilot-reader-mode-style';
            style.textContent = `
                body {
                    max-width: 740px !important;
                    margin: 40px auto !important;
                    padding: 0 24px !important;
                    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Liberation Sans", sans-serif !important;
                    font-size: 19px !important;
                    line-height: 1.75 !important;
                    background-color: #faf9f6 !important;
                    color: #1a1a1a !important;
                }
                @media (prefers-color-scheme: dark) {
                    body {
                        background-color: #1a1a1a !important;
                        color: #e6e6e6 !important;
                    }
                }
                header, nav, aside, footer, .sidebar, .ad, .advertisement, [role="banner"], [role="navigation"], [role="complementary"], .social-share, .comments, iframe, .cookie-banner, #cookie-banner {
                    display: none !important;
                }
                article, main, [role="main"] {
                    width: 100% !important;
                    margin: 0 !important;
                    padding: 0 !important;
                }
                img {
                    max-width: 100% !important;
                    height: auto !important;
                    border-radius: 8px !important;
                    margin: 16px 0 !important;
                }
                h1, h2, h3, h4 {
                    line-height: 1.3 !important;
                    margin-top: 24px !important;
                }
            `;
            document.head.appendChild(style);
        })();
    "#;

    if let Ok(c_script) = CString::new(script) {
        (b.webkit_web_view_evaluate_javascript)(
            webview,
            c_script.as_ptr(),
            -1,
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
    }
}

unsafe extern "C" fn on_webview_uri_notify(
    webview: *mut c_void,
    _pspec: *mut c_void,
    _user_data: *mut c_void,
) {
    let b = match BINDINGS.get() {
        Some(b) => b,
        None => return,
    };
    let entry = ACTIVE_ENTRY.load(Ordering::SeqCst);
    let uri_ptr = (b.webkit_web_view_get_uri)(webview);
    if !uri_ptr.is_null() {
        let uri_str = CStr::from_ptr(uri_ptr).to_string_lossy().to_string();
        if !entry.is_null() {
            (b.gtk_editable_set_text)(entry, uri_ptr);
        }
        if let Ok(mut mgr_guard) = TAB_MANAGER.lock() {
            if let Some(mgr) = mgr_guard.as_mut() {
                if let Some(active) = mgr.active_tab_mut() {
                    active.url = uri_str.clone();
                    active.is_start_page = false;
                }
            }
        }

        update_star_status_for_uri(&uri_str);

        let is_incog = IS_INCOGNITO.load(Ordering::SeqCst);
        if !is_incog && !uri_str.starts_with("about:") && !uri_str.starts_with("browser://") {
            let db_guard = BROWSER_DB.lock().ok();
            let prof_guard = ACTIVE_PROFILE_ID.lock().ok();
            let db = db_guard.as_ref().and_then(|g| g.as_ref());
            let prof_id = prof_guard.as_ref().and_then(|g| g.as_ref()).copied();
            if let (Some(db), Some(profile_id)) = (db, prof_id) {
                let hist_repo = HistoryRepository::new(db);
                let _ = hist_repo.record_visit(profile_id, &uri_str, None);
            }
        }
    }
}

unsafe fn refresh_sidebar_tabs() {
    let b = match BINDINGS.get() {
        Some(b) => b,
        None => return,
    };
    let pinned_box = ACTIVE_PINNED_BOX.load(Ordering::SeqCst);
    let normal_box = ACTIVE_NORMAL_BOX.load(Ordering::SeqCst);
    let sep = ACTIVE_SEPARATOR.load(Ordering::SeqCst);
    if pinned_box.is_null() || normal_box.is_null() || sep.is_null() {
        return;
    }

    // Clear pinned tabs box
    loop {
        let child = (b.gtk_widget_get_first_child)(pinned_box);
        if child.is_null() {
            break;
        }
        (b.gtk_box_remove)(pinned_box, child);
    }

    // Clear normal tabs box
    loop {
        let child = (b.gtk_widget_get_first_child)(normal_box);
        if child.is_null() {
            break;
        }
        (b.gtk_box_remove)(normal_box, child);
    }

    let is_collapsed = SIDEBAR_COLLAPSED.load(Ordering::SeqCst);

    let clicked_sig = CString::new("clicked").unwrap();
    let drag_end_sig = CString::new("drag-end").unwrap();

    let cb_select = std::mem::transmute::<unsafe extern "C" fn(*mut c_void, *mut c_void), GCallback>(
        on_tab_row_clicked,
    );
    let cb_close = std::mem::transmute::<unsafe extern "C" fn(*mut c_void, *mut c_void), GCallback>(
        on_tab_close_clicked,
    );
    let cb_pin_toggle = std::mem::transmute::<
        unsafe extern "C" fn(*mut c_void, *mut c_void),
        GCallback,
    >(on_tab_pin_toggle_clicked);
    let cb_drag_end = std::mem::transmute::<
        unsafe extern "C" fn(*mut c_void, libc::c_double, libc::c_double, *mut c_void),
        GCallback,
    >(on_tab_drag_end);
    let cb_sidebar_add = std::mem::transmute::<
        unsafe extern "C" fn(*mut c_void, *mut c_void),
        GCallback,
    >(on_sidebar_new_tab_clicked);

    let tab_manager_guard = match TAB_MANAGER.lock() {
        Ok(g) => g,
        Err(_) => return,
    };

    if let Some(mgr) = tab_manager_guard.as_ref() {
        let active_idx = mgr.active_index();
        let total = mgr.tabs().len();

        for (idx, tab) in mgr.tabs().iter().enumerate() {
            let is_active = idx == active_idx;

            // Favicon image
            let img = if tab.is_start_page {
                let c_home = CString::new("go-home-symbolic").unwrap();
                (b.gtk_image_new_from_icon_name)(c_home.as_ptr())
            } else if tab.favicon_ptr != 0 {
                (b.gtk_image_new_from_paintable)(tab.favicon_ptr as *mut c_void)
            } else {
                let fallback = if tab.is_incognito {
                    "weather-clear-night-symbolic"
                } else {
                    "globe-symbolic"
                };
                let c_fallback = CString::new(fallback).unwrap();
                (b.gtk_image_new_from_icon_name)(c_fallback.as_ptr())
            };
            (b.gtk_image_set_pixel_size)(img, 18);

            // Drag gesture for moving tab with mouse across the separator or reordering
            let drag_gesture = (b.gtk_gesture_drag_new)();
            (b.g_signal_connect_data)(
                drag_gesture,
                drag_end_sig.as_ptr(),
                cb_drag_end,
                idx as *mut c_void,
                std::ptr::null_mut(),
                0,
            );

            let tip_text = if tab.is_pinned {
                format!("📌 {}\n{}", tab.title, tab.url)
            } else {
                format!("{}\n{}", tab.title, tab.url)
            };
            let c_tip = CString::new(tip_text).unwrap();

            if is_collapsed {
                // Collapsed mode: ONLY favicon is visible
                let tab_btn = (b.gtk_button_new)();
                (b.gtk_button_set_has_frame)(tab_btn, 0);
                (b.gtk_button_set_child)(tab_btn, img);
                (b.gtk_widget_set_size_request)(tab_btn, 38, 38);
                (b.gtk_widget_set_tooltip_text)(tab_btn, c_tip.as_ptr());

                if is_active {
                    let active_class = CString::new("suggested-action").unwrap();
                    (b.gtk_widget_add_css_class)(tab_btn, active_class.as_ptr());
                }

                (b.g_signal_connect_data)(
                    tab_btn,
                    clicked_sig.as_ptr(),
                    cb_select,
                    idx as *mut c_void,
                    std::ptr::null_mut(),
                    0,
                );
                (b.gtk_widget_add_controller)(tab_btn, drag_gesture);

                if tab.is_pinned {
                    (b.gtk_box_append)(pinned_box, tab_btn);
                } else {
                    (b.gtk_box_append)(normal_box, tab_btn);
                }
            } else {
                // Expanded mode: favicon + title + action buttons
                let row_box = (b.gtk_box_new)(0, 2);
                (b.gtk_widget_set_hexpand)(row_box, 1);
                (b.gtk_widget_set_margin_start)(row_box, 2);
                (b.gtk_widget_set_margin_end)(row_box, 2);
                (b.gtk_widget_set_margin_top)(row_box, 1);
                (b.gtk_widget_set_margin_bottom)(row_box, 1);

                let tab_btn = (b.gtk_button_new)();
                (b.gtk_button_set_has_frame)(tab_btn, 0);
                (b.gtk_widget_set_hexpand)(tab_btn, 1);

                let inner = (b.gtk_box_new)(0, 6);
                (b.gtk_box_append)(inner, img);

                let c_title = CString::new(tab.title.clone()).unwrap();
                let lbl_tab = (b.gtk_label_new)(c_title.as_ptr());
                (b.gtk_label_set_xalign)(lbl_tab, 0.0);
                (b.gtk_label_set_ellipsize)(lbl_tab, 3); // PANGO_ELLIPSIZE_END
                (b.gtk_widget_set_hexpand)(lbl_tab, 1);
                (b.gtk_box_append)(inner, lbl_tab);

                (b.gtk_button_set_child)(tab_btn, inner);
                (b.gtk_widget_set_tooltip_text)(tab_btn, c_tip.as_ptr());

                if is_active {
                    let active_class = CString::new("suggested-action").unwrap();
                    (b.gtk_widget_add_css_class)(tab_btn, active_class.as_ptr());
                }

                (b.g_signal_connect_data)(
                    tab_btn,
                    clicked_sig.as_ptr(),
                    cb_select,
                    idx as *mut c_void,
                    std::ptr::null_mut(),
                    0,
                );
                (b.gtk_widget_add_controller)(tab_btn, drag_gesture);
                (b.gtk_box_append)(row_box, tab_btn);

                if tab.is_pinned {
                    // Small unpin button (or drag below separator)
                    let pin_icon = CString::new("view-pin-symbolic").unwrap();
                    let btn_unpin = (b.gtk_button_new_from_icon_name)(pin_icon.as_ptr());
                    (b.gtk_button_set_has_frame)(btn_unpin, 0);
                    let unpin_tip = CString::new("Unpin Tab (or drag below separator)").unwrap();
                    (b.gtk_widget_set_tooltip_text)(btn_unpin, unpin_tip.as_ptr());
                    (b.g_signal_connect_data)(
                        btn_unpin,
                        clicked_sig.as_ptr(),
                        cb_pin_toggle,
                        idx as *mut c_void,
                        std::ptr::null_mut(),
                        0,
                    );
                    (b.gtk_box_append)(row_box, btn_unpin);
                    (b.gtk_box_append)(pinned_box, row_box);
                } else {
                    // Normal tab: Close button (if > 1 tab)
                    if total > 1 {
                        let close_icon = CString::new("window-close-symbolic").unwrap();
                        let btn_close = (b.gtk_button_new_from_icon_name)(close_icon.as_ptr());
                        (b.gtk_button_set_has_frame)(btn_close, 0);
                        let close_tip = CString::new("Close Tab").unwrap();
                        (b.gtk_widget_set_tooltip_text)(btn_close, close_tip.as_ptr());
                        (b.g_signal_connect_data)(
                            btn_close,
                            clicked_sig.as_ptr(),
                            cb_close,
                            idx as *mut c_void,
                            std::ptr::null_mut(),
                            0,
                        );
                        (b.gtk_box_append)(row_box, btn_close);
                    }
                    (b.gtk_box_append)(normal_box, row_box);
                }
            }
        }

        // Plus button: situated under the horizontal separator (in normal_box).
        // If no normal tabs are open, it sits right under the separator.
        // If normal tabs are open, it sits directly below them.
        let btn_add = (b.gtk_button_new)();
        (b.gtk_button_set_has_frame)(btn_add, 0);

        if is_collapsed {
            let add_icon = CString::new("list-add-symbolic").unwrap();
            let add_img = (b.gtk_image_new_from_icon_name)(add_icon.as_ptr());
            (b.gtk_image_set_pixel_size)(add_img, 16);
            (b.gtk_button_set_child)(btn_add, add_img);
            (b.gtk_widget_set_size_request)(btn_add, 38, 38);
            let add_tip = CString::new("Open New Tab").unwrap();
            (b.gtk_widget_set_tooltip_text)(btn_add, add_tip.as_ptr());
        } else {
            let add_inner = (b.gtk_box_new)(0, 6);
            let add_icon = CString::new("list-add-symbolic").unwrap();
            let add_img = (b.gtk_image_new_from_icon_name)(add_icon.as_ptr());
            (b.gtk_image_set_pixel_size)(add_img, 16);
            let add_lbl = (b.gtk_label_new)(CString::new("New Tab").unwrap().as_ptr());
            (b.gtk_label_set_xalign)(add_lbl, 0.0);
            (b.gtk_box_append)(add_inner, add_img);
            (b.gtk_box_append)(add_inner, add_lbl);
            (b.gtk_button_set_child)(btn_add, add_inner);
            (b.gtk_widget_set_hexpand)(btn_add, 1);
            let add_tip = CString::new("Open New Tab").unwrap();
            (b.gtk_widget_set_tooltip_text)(btn_add, add_tip.as_ptr());
        }

        (b.g_signal_connect_data)(
            btn_add,
            clicked_sig.as_ptr(),
            cb_sidebar_add,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            0,
        );
        (b.gtk_box_append)(normal_box, btn_add);
    }
}

unsafe extern "C" fn on_tab_row_clicked(_button: *mut c_void, user_data: *mut c_void) {
    let idx = user_data as usize;
    let b = match BINDINGS.get() {
        Some(b) => b,
        None => return,
    };
    let webview = ACTIVE_WEBVIEW.load(Ordering::SeqCst);
    let entry = ACTIVE_ENTRY.load(Ordering::SeqCst);

    let mut url_to_load = None;
    let mut is_start = false;
    if let Ok(mut mgr_guard) = TAB_MANAGER.lock() {
        if let Some(mgr) = mgr_guard.as_mut() {
            // Save active tab's current URL before switching
            if !webview.is_null() {
                let current_uri_ptr = (b.webkit_web_view_get_uri)(webview);
                if !current_uri_ptr.is_null() {
                    let cur_uri = CStr::from_ptr(current_uri_ptr)
                        .to_string_lossy()
                        .to_string();
                    if let Some(active) = mgr.active_tab_mut() {
                        if !active.is_start_page {
                            active.url = cur_uri.clone();
                            let title_ptr = (b.webkit_web_view_get_title)(webview);
                            let page_title = if !title_ptr.is_null() {
                                let t = CStr::from_ptr(title_ptr)
                                    .to_string_lossy()
                                    .trim()
                                    .to_string();
                                if t.is_empty() {
                                    None
                                } else {
                                    Some(t)
                                }
                            } else {
                                None
                            };

                            active.title = if active.is_incognito {
                                "Incognito Mode".to_string()
                            } else if let Some(t) = page_title {
                                t
                            } else if cur_uri.is_empty() || cur_uri == "about:blank" {
                                "New Tab".to_string()
                            } else {
                                cur_uri.clone()
                            };
                        }
                    }
                }
            }

            if mgr.switch_tab(idx) {
                if let Some(tab) = mgr.active_tab() {
                    is_start = tab.is_start_page
                        || tab.url == "about:blank"
                        || tab.url == "browser://start";
                    url_to_load = Some(tab.url.clone());
                }
            }
        }
    }

    if is_start {
        switch_to_start_page();
    } else if let Some(url) = url_to_load {
        switch_to_webview();
        if !webview.is_null() {
            if let Ok(c_url) = CString::new(url.clone()) {
                (b.webkit_web_view_load_uri)(webview, c_url.as_ptr());
            }
        }
        if !entry.is_null() {
            if let Ok(c_url) = CString::new(url) {
                (b.gtk_editable_set_text)(entry, c_url.as_ptr());
            }
        }
    }

    refresh_sidebar_tabs();
}

unsafe extern "C" fn on_tab_close_clicked(_button: *mut c_void, user_data: *mut c_void) {
    let idx = user_data as usize;
    let b = match BINDINGS.get() {
        Some(b) => b,
        None => return,
    };
    let webview = ACTIVE_WEBVIEW.load(Ordering::SeqCst);
    let entry = ACTIVE_ENTRY.load(Ordering::SeqCst);

    let mut url_to_load = None;
    let mut is_start = false;
    if let Ok(mut mgr_guard) = TAB_MANAGER.lock() {
        if let Some(mgr) = mgr_guard.as_mut() {
            mgr.close_tab(idx);
            if let Some(tab) = mgr.active_tab() {
                is_start =
                    tab.is_start_page || tab.url == "about:blank" || tab.url == "browser://start";
                url_to_load = Some(tab.url.clone());
            }
        }
    }

    if is_start {
        switch_to_start_page();
    } else if let Some(url) = url_to_load {
        switch_to_webview();
        if !webview.is_null() {
            if let Ok(c_url) = CString::new(url.clone()) {
                (b.webkit_web_view_load_uri)(webview, c_url.as_ptr());
            }
        }
        if !entry.is_null() {
            if let Ok(c_url) = CString::new(url) {
                (b.gtk_editable_set_text)(entry, c_url.as_ptr());
            }
        }
    }

    refresh_sidebar_tabs();
}

unsafe extern "C" fn on_sidebar_new_tab_clicked(_button: *mut c_void, _user_data: *mut c_void) {
    if let Ok(mut mgr_guard) = TAB_MANAGER.lock() {
        if let Some(mgr) = mgr_guard.as_mut() {
            mgr.new_tab();
            if let Some(active) = mgr.active_tab_mut() {
                active.is_start_page = true;
                active.url = "about:blank".to_string();
                active.title = "Start Page".to_string();
            }
        }
    }

    switch_to_start_page();
    refresh_sidebar_tabs();
}

unsafe extern "C" fn on_toggle_sidebar_clicked(_button: *mut c_void, _user_data: *mut c_void) {
    let sidebar = ACTIVE_SIDEBAR_BOX.load(Ordering::SeqCst);
    let paned = ACTIVE_PANED.load(Ordering::SeqCst);
    if sidebar.is_null() || paned.is_null() {
        return;
    }
    let b = match BINDINGS.get() {
        Some(b) => b,
        None => return,
    };

    let is_collapsed = !SIDEBAR_COLLAPSED.load(Ordering::SeqCst);
    SIDEBAR_COLLAPSED.store(is_collapsed, Ordering::SeqCst);

    if is_collapsed {
        (b.gtk_paned_set_position)(paned, MIN_COLLAPSED_SIDEBAR_WIDTH);
        (b.gtk_widget_set_size_request)(sidebar, MIN_COLLAPSED_SIDEBAR_WIDTH, -1);
    } else {
        (b.gtk_paned_set_position)(paned, 220);
        (b.gtk_widget_set_size_request)(sidebar, MIN_COLLAPSED_SIDEBAR_WIDTH, -1);
    }

    refresh_sidebar_tabs();
}

unsafe extern "C" fn on_new_tab_clicked(_button: *mut c_void, _user_data: *mut c_void) {
    let popover = ACTIVE_POPOVER.load(Ordering::SeqCst);
    if !popover.is_null() {
        if let Some(b) = BINDINGS.get() {
            (b.gtk_popover_popdown)(popover);
        }
    }
    on_sidebar_new_tab_clicked(std::ptr::null_mut(), std::ptr::null_mut());
}

unsafe extern "C" fn on_new_incognito_clicked(_button: *mut c_void, _user_data: *mut c_void) {
    let popover = ACTIVE_POPOVER.load(Ordering::SeqCst);
    if !popover.is_null() {
        if let Some(b) = BINDINGS.get() {
            (b.gtk_popover_popdown)(popover);
        }
    }
    let _ = std::process::Command::new("vingilot-ui")
        .arg("--incognito")
        .spawn();
}

unsafe fn update_zoom_display(zoom: f64) {
    let b = match BINDINGS.get() {
        Some(b) => b,
        None => return,
    };
    let lbl = ACTIVE_ZOOM_LABEL.load(Ordering::SeqCst);
    let lens_btn = ACTIVE_LENS_BTN.load(Ordering::SeqCst);
    let pct_text = format!("{:.0}%", zoom * 100.0);

    if !lbl.is_null() {
        if let Ok(c_text) = CString::new(pct_text.clone()) {
            (b.gtk_label_set_text)(lbl, c_text.as_ptr());
        }
    }

    if !lens_btn.is_null() {
        let lens_label = format!("🔍 {}", pct_text);
        if let Ok(c_label) = CString::new(lens_label) {
            (b.gtk_menu_button_set_label)(lens_btn, c_label.as_ptr());
        }
    }
}

unsafe extern "C" fn on_zoom_in_clicked(_button: *mut c_void, _user_data: *mut c_void) {
    let webview = ACTIVE_WEBVIEW.load(Ordering::SeqCst);
    if webview.is_null() {
        return;
    }
    if let Some(b) = BINDINGS.get() {
        let current = (b.webkit_web_view_get_zoom_level)(webview);
        let new_zoom = (current + 0.1).min(3.0);
        (b.webkit_web_view_set_zoom_level)(webview, new_zoom);
        update_zoom_display(new_zoom);
    }
}

unsafe extern "C" fn on_zoom_out_clicked(_button: *mut c_void, _user_data: *mut c_void) {
    let webview = ACTIVE_WEBVIEW.load(Ordering::SeqCst);
    if webview.is_null() {
        return;
    }
    if let Some(b) = BINDINGS.get() {
        let current = (b.webkit_web_view_get_zoom_level)(webview);
        let new_zoom = (current - 0.1).max(0.5);
        (b.webkit_web_view_set_zoom_level)(webview, new_zoom);
        update_zoom_display(new_zoom);
    }
}

unsafe extern "C" fn on_zoom_reset_clicked(_button: *mut c_void, _user_data: *mut c_void) {
    let webview = ACTIVE_WEBVIEW.load(Ordering::SeqCst);
    if webview.is_null() {
        return;
    }
    if let Some(b) = BINDINGS.get() {
        (b.webkit_web_view_set_zoom_level)(webview, 1.0);
        update_zoom_display(1.0);
    }
}

unsafe extern "C" fn on_lens_popover_closed(_popover: *mut c_void, _user_data: *mut c_void) {
    let b = match BINDINGS.get() {
        Some(b) => b,
        None => return,
    };
    let lens_btn = ACTIVE_LENS_BTN.load(Ordering::SeqCst);
    if !lens_btn.is_null() {
        (b.gtk_menu_button_set_label)(lens_btn, std::ptr::null());
        let icon_name = CString::new("system-search-symbolic").unwrap();
        (b.gtk_menu_button_set_icon_name)(lens_btn, icon_name.as_ptr());
    }
}

unsafe extern "C" fn on_lens_button_notify_active(
    _btn: *mut c_void,
    _pspec: *mut c_void,
    _user_data: *mut c_void,
) {
    let b = match BINDINGS.get() {
        Some(b) => b,
        None => return,
    };
    let webview = ACTIVE_WEBVIEW.load(Ordering::SeqCst);
    let zoom = if !webview.is_null() {
        (b.webkit_web_view_get_zoom_level)(webview)
    } else {
        1.0
    };
    update_zoom_display(zoom);
}

unsafe extern "C" fn on_history_clicked(_button: *mut c_void, _user_data: *mut c_void) {
    let popover = ACTIVE_POPOVER.load(Ordering::SeqCst);
    if !popover.is_null() {
        if let Some(b) = BINDINGS.get() {
            (b.gtk_popover_popdown)(popover);
        }
    }
    let webview = ACTIVE_WEBVIEW.load(Ordering::SeqCst);
    let entry = ACTIVE_ENTRY.load(Ordering::SeqCst);
    if webview.is_null() {
        return;
    }
    if let Some(b) = BINDINGS.get() {
        let uri = CString::new("https://html.duckduckgo.com").unwrap();
        (b.webkit_web_view_load_uri)(webview, uri.as_ptr());
        if !entry.is_null() {
            (b.gtk_editable_set_text)(entry, uri.as_ptr());
        }
    }
}

#[allow(dead_code)]
unsafe extern "C" fn on_preferences_clicked(_button: *mut c_void, _user_data: *mut c_void) {
    let popover = ACTIVE_POPOVER.load(Ordering::SeqCst);
    if !popover.is_null() {
        if let Some(b) = BINDINGS.get() {
            (b.gtk_popover_popdown)(popover);
        }
    }

    let b = match BINDINGS.get() {
        Some(b) => b,
        None => return,
    };
    let parent = ACTIVE_WINDOW.load(Ordering::SeqCst);

    let prefs = (b.adw_preferences_window_new)();
    if !parent.is_null() {
        (b.gtk_window_set_transient_for)(prefs, parent);
        (b.gtk_window_set_modal)(prefs, 1);
    }
    let prefs_title = CString::new("Preferences").unwrap();
    (b.gtk_window_set_title)(prefs, prefs_title.as_ptr());
    (b.gtk_window_set_default_size)(prefs, 680, 560);

    // Page 1: General
    let page_gen = (b.adw_preferences_page_new)();
    let gen_title = CString::new("General").unwrap();
    let gen_icon = CString::new("preferences-system-symbolic").unwrap();
    (b.adw_preferences_page_set_title)(page_gen, gen_title.as_ptr());
    (b.adw_preferences_page_set_icon_name)(page_gen, gen_icon.as_ptr());

    // Appearance Group
    let grp_appear = (b.adw_preferences_group_new)();
    let grp_appear_title = CString::new("Appearance & Themes").unwrap();
    (b.adw_preferences_group_set_title)(grp_appear, grp_appear_title.as_ptr());

    let row_dark = (b.adw_switch_row_new)();
    (b.adw_preferences_row_set_title)(
        row_dark,
        CString::new("True Black OLED Theme").unwrap().as_ptr(),
    );
    (b.adw_action_row_set_subtitle)(
        row_dark,
        CString::new("Deep pure black theme optimized for OLED displays")
            .unwrap()
            .as_ptr(),
    );
    (b.adw_switch_row_set_active)(row_dark, 1);
    (b.adw_preferences_group_add)(grp_appear, row_dark);

    let row_reader = (b.adw_switch_row_new)();
    (b.adw_preferences_row_set_title)(
        row_reader,
        CString::new("Auto Reader Mode").unwrap().as_ptr(),
    );
    (b.adw_action_row_set_subtitle)(
        row_reader,
        CString::new("Automatically offer distraction-free article reader view")
            .unwrap()
            .as_ptr(),
    );
    (b.adw_switch_row_set_active)(row_reader, 1);
    (b.adw_preferences_group_add)(grp_appear, row_reader);

    (b.adw_preferences_page_add)(page_gen, grp_appear);

    // Search & Navigation Group
    let grp_nav = (b.adw_preferences_group_new)();
    (b.adw_preferences_group_set_title)(
        grp_nav,
        CString::new("Search Engine & Navigation").unwrap().as_ptr(),
    );

    let row_search = (b.adw_action_row_new)();
    (b.adw_preferences_row_set_title)(
        row_search,
        CString::new("Default Search Engine").unwrap().as_ptr(),
    );
    (b.adw_action_row_set_subtitle)(
        row_search,
        CString::new("DuckDuckGo (Privacy Encrypted)")
            .unwrap()
            .as_ptr(),
    );
    (b.adw_preferences_group_add)(grp_nav, row_search);

    let row_restore = (b.adw_switch_row_new)();
    (b.adw_preferences_row_set_title)(
        row_restore,
        CString::new("Restore Tabs on Launch").unwrap().as_ptr(),
    );
    (b.adw_action_row_set_subtitle)(
        row_restore,
        CString::new("Reopen previously open tabs on startup")
            .unwrap()
            .as_ptr(),
    );
    (b.adw_switch_row_set_active)(row_restore, 1);
    (b.adw_preferences_group_add)(grp_nav, row_restore);

    (b.adw_preferences_page_add)(page_gen, grp_nav);

    // Graphics & Performance Group
    let grp_gfx = (b.adw_preferences_group_new)();
    (b.adw_preferences_group_set_title)(
        grp_gfx,
        CString::new("Hardware & Rendering").unwrap().as_ptr(),
    );

    let row_vulkan = (b.adw_switch_row_new)();
    (b.adw_preferences_row_set_title)(
        row_vulkan,
        CString::new("Vulkan Hardware Acceleration")
            .unwrap()
            .as_ptr(),
    );
    (b.adw_action_row_set_subtitle)(
        row_vulkan,
        CString::new("Accelerate GTK Scene Kit (GSK) with Vulkan graphics pipeline")
            .unwrap()
            .as_ptr(),
    );
    (b.adw_switch_row_set_active)(row_vulkan, 1);
    (b.adw_preferences_group_add)(grp_gfx, row_vulkan);

    let row_wayland = (b.adw_switch_row_new)();
    (b.adw_preferences_row_set_title)(
        row_wayland,
        CString::new("Native Wayland Compositing").unwrap().as_ptr(),
    );
    (b.adw_action_row_set_subtitle)(
        row_wayland,
        CString::new("Direct fractional scaling and subpixel presentation")
            .unwrap()
            .as_ptr(),
    );
    (b.adw_switch_row_set_active)(row_wayland, 1);
    (b.adw_preferences_group_add)(grp_gfx, row_wayland);

    (b.adw_preferences_page_add)(page_gen, grp_gfx);
    (b.adw_preferences_window_add)(prefs, page_gen);

    // Page 2: Privacy & AI
    let page_priv = (b.adw_preferences_page_new)();
    (b.adw_preferences_page_set_title)(page_priv, CString::new("Privacy & AI").unwrap().as_ptr());
    (b.adw_preferences_page_set_icon_name)(
        page_priv,
        CString::new("security-high-symbolic").unwrap().as_ptr(),
    );

    let grp_priv = (b.adw_preferences_group_new)();
    (b.adw_preferences_group_set_title)(
        grp_priv,
        CString::new("Privacy & Protection").unwrap().as_ptr(),
    );

    let row_trackers = (b.adw_switch_row_new)();
    (b.adw_preferences_row_set_title)(
        row_trackers,
        CString::new("Enhanced Tracking Protection")
            .unwrap()
            .as_ptr(),
    );
    (b.adw_action_row_set_subtitle)(
        row_trackers,
        CString::new("Block third-party tracking cookies, cryptominers, and fingerprinters")
            .unwrap()
            .as_ptr(),
    );
    (b.adw_switch_row_set_active)(row_trackers, 1);
    (b.adw_preferences_group_add)(grp_priv, row_trackers);

    let row_incog_iso = (b.adw_switch_row_new)();
    (b.adw_preferences_row_set_title)(
        row_incog_iso,
        CString::new("Strict Incognito Isolation").unwrap().as_ptr(),
    );
    (b.adw_action_row_set_subtitle)(
        row_incog_iso,
        CString::new("Zero cache and ephemeral memory storage in private sessions")
            .unwrap()
            .as_ptr(),
    );
    (b.adw_switch_row_set_active)(row_incog_iso, 1);
    (b.adw_preferences_group_add)(grp_priv, row_incog_iso);

    (b.adw_preferences_page_add)(page_priv, grp_priv);

    let grp_ai = (b.adw_preferences_group_new)();
    (b.adw_preferences_group_set_title)(
        grp_ai,
        CString::new("Local AI Assistant").unwrap().as_ptr(),
    );

    let row_ai_provider = (b.adw_action_row_new)();
    (b.adw_preferences_row_set_title)(
        row_ai_provider,
        CString::new("AI Endpoint").unwrap().as_ptr(),
    );
    (b.adw_action_row_set_subtitle)(
        row_ai_provider,
        CString::new("Ollama / OpenAI API (http://localhost:11434/v1)")
            .unwrap()
            .as_ptr(),
    );
    (b.adw_preferences_group_add)(grp_ai, row_ai_provider);

    let row_ai_model = (b.adw_action_row_new)();
    (b.adw_preferences_row_set_title)(
        row_ai_model,
        CString::new("Default Model").unwrap().as_ptr(),
    );
    (b.adw_action_row_set_subtitle)(
        row_ai_model,
        CString::new("llama3.2:latest").unwrap().as_ptr(),
    );
    (b.adw_preferences_group_add)(grp_ai, row_ai_model);

    (b.adw_preferences_page_add)(page_priv, grp_ai);
    (b.adw_preferences_window_add)(prefs, page_priv);

    (b.gtk_window_present)(prefs);
}

unsafe extern "C" fn on_about_clicked(_button: *mut c_void, _user_data: *mut c_void) {
    let popover = ACTIVE_POPOVER.load(Ordering::SeqCst);
    if !popover.is_null() {
        if let Some(b) = BINDINGS.get() {
            (b.gtk_popover_popdown)(popover);
        }
    }

    let b = match BINDINGS.get() {
        Some(b) => b,
        None => return,
    };
    let parent = ACTIVE_WINDOW.load(Ordering::SeqCst);

    let dialog = (b.adw_about_dialog_new)();
    (b.adw_about_dialog_set_application_name)(dialog, CString::new("Vingilot").unwrap().as_ptr());
    (b.adw_about_dialog_set_application_icon)(
        dialog,
        CString::new("io.github.antux912012.Vingilot")
            .unwrap()
            .as_ptr(),
    );
    (b.adw_about_dialog_set_version)(dialog, CString::new("0.1.2-b").unwrap().as_ptr());
    (b.adw_about_dialog_set_developer_name)(
        dialog,
        CString::new("Antonio & The Vingilot Project Team")
            .unwrap()
            .as_ptr(),
    );
    (b.adw_about_dialog_set_comments)(
        dialog,
        CString::new("Modern, privacy-first web browser powered by Libadwaita, WebKitGTK 6.0, and hardware-accelerated Vulkan & Wayland rendering.").unwrap().as_ptr(),
    );
    (b.adw_about_dialog_set_website)(
        dialog,
        CString::new("https://github.com/Antux912012/Vingilot")
            .unwrap()
            .as_ptr(),
    );
    (b.adw_about_dialog_set_issue_url)(
        dialog,
        CString::new("https://github.com/Antux912012/Vingilot/issues")
            .unwrap()
            .as_ptr(),
    );

    (b.adw_dialog_present)(dialog, parent);
}

unsafe extern "C" fn on_toggle_right_menu_clicked(_button: *mut c_void, _user_data: *mut c_void) {
    let side_menu = ACTIVE_RIGHT_SIDEBAR.load(Ordering::SeqCst);
    let right_paned = ACTIVE_RIGHT_PANED.load(Ordering::SeqCst);
    let win = ACTIVE_WINDOW.load(Ordering::SeqCst);
    if !side_menu.is_null() {
        if let Some(b) = BINDINGS.get() {
            let vis = (b.gtk_widget_get_visible)(side_menu);
            let new_vis = if vis == 0 { 1 } else { 0 };
            (b.gtk_widget_set_visible)(side_menu, new_vis);
            if new_vis == 1 && !right_paned.is_null() && !win.is_null() {
                let win_w = (b.gtk_widget_get_width)(win);
                let target_pos = if win_w > 600 {
                    win_w - 380
                } else {
                    (win_w * 2) / 3
                };
                (b.gtk_paned_set_position)(right_paned, target_pos);
            }
        }
    }
}

unsafe extern "C" fn on_close_right_menu_clicked(_button: *mut c_void, _user_data: *mut c_void) {
    let side_menu = ACTIVE_RIGHT_SIDEBAR.load(Ordering::SeqCst);
    if !side_menu.is_null() {
        if let Some(b) = BINDINGS.get() {
            (b.gtk_widget_set_visible)(side_menu, 0);
        }
    }
}

unsafe extern "C" fn on_save_ai_settings_clicked(_button: *mut c_void, _user_data: *mut c_void) {
    let b = match BINDINGS.get() {
        Some(b) => b,
        None => return,
    };
    let base_url_entry = ACTIVE_AI_BASE_URL_ENTRY.load(Ordering::SeqCst);
    let model_entry = ACTIVE_AI_MODEL_ENTRY.load(Ordering::SeqCst);
    let status_lbl = ACTIVE_AI_STATUS_LABEL.load(Ordering::SeqCst);

    let base_url = if !base_url_entry.is_null() {
        let ptr = (b.gtk_editable_get_text)(base_url_entry);
        if !ptr.is_null() {
            CStr::from_ptr(ptr).to_string_lossy().to_string()
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let model = if !model_entry.is_null() {
        let ptr = (b.gtk_editable_get_text)(model_entry);
        if !ptr.is_null() {
            CStr::from_ptr(ptr).to_string_lossy().to_string()
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    println!(
        "[Vingilot] AI configuration saved: endpoint='{}', model='{}'",
        base_url, model
    );

    if !status_lbl.is_null() {
        let status_msg = CString::new("✓ AI Configuration Saved & Active!").unwrap();
        (b.gtk_label_set_text)(status_lbl, status_msg.as_ptr());
    }
}

unsafe fn navigate_to_url(url: &str) {
    let b = match BINDINGS.get() {
        Some(b) => b,
        None => return,
    };
    let trimmed = url.trim();
    if trimmed == "about:blank" || trimmed == "browser://start" {
        if let Ok(mut mgr_guard) = TAB_MANAGER.lock() {
            if let Some(mgr) = mgr_guard.as_mut() {
                if let Some(active) = mgr.active_tab_mut() {
                    active.is_start_page = true;
                    active.url = "about:blank".to_string();
                    active.title = "Start Page".to_string();
                }
            }
        }
        switch_to_start_page();
        refresh_sidebar_tabs();
        return;
    }

    switch_to_webview();
    let webview = ACTIVE_WEBVIEW.load(Ordering::SeqCst);
    let entry = ACTIVE_ENTRY.load(Ordering::SeqCst);
    if !webview.is_null() {
        if let Ok(c_url) = CString::new(trimmed) {
            (b.webkit_web_view_load_uri)(webview, c_url.as_ptr());
            if !entry.is_null() {
                (b.gtk_editable_set_text)(entry, c_url.as_ptr());
            }
        }
    }
    if let Ok(mut mgr_guard) = TAB_MANAGER.lock() {
        if let Some(mgr) = mgr_guard.as_mut() {
            if let Some(active) = mgr.active_tab_mut() {
                active.is_start_page = false;
                active.url = trimmed.to_string();
            }
        }
    }
    refresh_sidebar_tabs();
}

unsafe fn apply_css_to_display(css: &str) {
    let b = match BINDINGS.get() {
        Some(b) => b,
        None => return,
    };
    let display = (b.gdk_display_get_default)();
    if !display.is_null() {
        let provider = (b.gtk_css_provider_new)();
        if let Ok(c_css) = CString::new(css) {
            (b.gtk_css_provider_load_from_string)(provider, c_css.as_ptr());
            (b.gtk_style_context_add_provider_for_display)(display, provider, 800);
        }
    }
}

unsafe extern "C" fn on_search_ddg_clicked(_b: *mut c_void, _u: *mut c_void) {
    navigate_to_url("https://duckduckgo.com");
}
unsafe extern "C" fn on_search_google_clicked(_b: *mut c_void, _u: *mut c_void) {
    navigate_to_url("https://www.google.com");
}
unsafe extern "C" fn on_search_brave_clicked(_b: *mut c_void, _u: *mut c_void) {
    navigate_to_url("https://search.brave.com");
}
unsafe extern "C" fn on_search_bing_clicked(_b: *mut c_void, _u: *mut c_void) {
    navigate_to_url("https://www.bing.com");
}
unsafe extern "C" fn on_search_ecosia_clicked(_b: *mut c_void, _u: *mut c_void) {
    navigate_to_url("https://www.ecosia.org");
}

unsafe extern "C" fn on_theme_system_clicked(_b: *mut c_void, _u: *mut c_void) {
    apply_css_to_display(crate::theme::base_css());
}
unsafe extern "C" fn on_theme_light_clicked(_b: *mut c_void, _u: *mut c_void) {
    apply_css_to_display("window, .background, headerbar, .sidebar, .right-sidebar { background-color: #fafafa !important; color: #111111 !important; }");
}
unsafe extern "C" fn on_theme_dark_clicked(_b: *mut c_void, _u: *mut c_void) {
    apply_css_to_display("window, .background, headerbar, .sidebar, .right-sidebar { background-color: #242424 !important; color: #ffffff !important; }");
}
unsafe extern "C" fn on_theme_true_black_clicked(_b: *mut c_void, _u: *mut c_void) {
    apply_css_to_display(crate::theme::AppTheme::TrueBlack.custom_css().unwrap_or(""));
}
unsafe extern "C" fn on_theme_incognito_blue_clicked(_b: *mut c_void, _u: *mut c_void) {
    apply_css_to_display(crate::theme::incognito_css());
}

unsafe fn make_switch_row(
    b: &GtkBindings,
    title: &str,
    is_active: bool,
) -> (*mut c_void, *mut c_void) {
    let row = (b.gtk_box_new)(0, 8);
    (b.gtk_widget_set_margin_top)(row, 3);
    (b.gtk_widget_set_margin_bottom)(row, 3);
    let lbl = (b.gtk_label_new)(CString::new(title).unwrap().as_ptr());
    (b.gtk_label_set_xalign)(lbl, 0.0);
    (b.gtk_widget_set_hexpand)(lbl, 1);
    let sw = (b.gtk_switch_new)();
    (b.gtk_switch_set_active)(sw, if is_active { 1 } else { 0 });
    (b.gtk_box_append)(row, lbl);
    (b.gtk_box_append)(row, sw);
    (row, sw)
}

unsafe fn init_browser_storage(is_incognito: bool) {
    if is_incognito {
        let db = Database::in_memory().expect("In-memory SQLite database initialization failed");
        let pm = vingilot_core::profile_manager::ProfileManager::new(&db, "/tmp");
        let profiles = pm
            .initialize_default_profiles()
            .expect("Default profile creation failed");
        let profile_id = profiles[0].id;
        let sm = SearchEngineManager::new(&db);
        let _ = sm.initialize_defaults();

        let mut model = StartPageModel::new();
        let _ = model.reload(&db, profile_id);

        if let Ok(mut db_guard) = BROWSER_DB.lock() {
            *db_guard = Some(db);
        }
        if let Ok(mut prof_guard) = ACTIVE_PROFILE_ID.lock() {
            *prof_guard = Some(profile_id);
        }
        if let Ok(mut model_guard) = START_PAGE_MODEL.lock() {
            *model_guard = Some(model);
        }
        return;
    }

    let data_dir = default_data_dir();
    let db_path = default_db_path();
    let _ = std::fs::create_dir_all(&data_dir);
    let _ = std::fs::create_dir_all(default_previews_dir());

    let db = Database::open(&db_path).expect("Persistent SQLite database initialization failed");
    let pm = vingilot_core::profile_manager::ProfileManager::new(&db, &data_dir);
    let profiles = pm
        .initialize_default_profiles()
        .expect("Default profile creation failed");
    let profile_id = profiles[0].id;

    let sm = SearchEngineManager::new(&db);
    let _ = sm.initialize_defaults();

    let bm_store = BookmarkStore::new(&db);
    if let Ok(root_items) = bm_store.get_root_items() {
        if root_items.is_empty() {
            if let Ok(folder) = bm_store.create_folder("Daily Favorites", None) {
                let _ = bm_store.create_bookmark(
                    "Rust Language",
                    "https://www.rust-lang.org",
                    Some(folder.id),
                );
                let _ = bm_store.create_bookmark(
                    "GNOME / Libadwaita",
                    "https://gnome.pages.gitlab.gnome.org/libadwaita/doc/",
                    Some(folder.id),
                );
            }
            let _ = bm_store.create_bookmark("DuckDuckGo", "https://duckduckgo.com", None);
            let _ = bm_store.create_bookmark("GitHub", "https://github.com", None);
            let _ = bm_store.create_bookmark("Hacker News", "https://news.ycombinator.com", None);
        }
    }

    let mut model = StartPageModel::new();
    let _ = model.reload(&db, profile_id);

    if let Ok(mut db_guard) = BROWSER_DB.lock() {
        *db_guard = Some(db);
    }
    if let Ok(mut prof_guard) = ACTIVE_PROFILE_ID.lock() {
        *prof_guard = Some(profile_id);
    }
    if let Ok(mut model_guard) = START_PAGE_MODEL.lock() {
        *model_guard = Some(model);
    }
}

unsafe fn switch_to_start_page() {
    let b = match BINDINGS.get() {
        Some(b) => b,
        None => return,
    };
    let stack = ACTIVE_CONTENT_STACK.load(Ordering::SeqCst);
    if !stack.is_null() {
        let name = CString::new("start_page").unwrap();
        (b.gtk_stack_set_visible_child_name)(stack, name.as_ptr());
    }
    let entry = ACTIVE_ENTRY.load(Ordering::SeqCst);
    if !entry.is_null() {
        let c_empty = CString::new("").unwrap();
        (b.gtk_editable_set_text)(entry, c_empty.as_ptr());
    }
    queue_refresh_start_page();
}

unsafe fn switch_to_webview() {
    let b = match BINDINGS.get() {
        Some(b) => b,
        None => return,
    };
    let stack = ACTIVE_CONTENT_STACK.load(Ordering::SeqCst);
    if !stack.is_null() {
        let name = CString::new("webview").unwrap();
        (b.gtk_stack_set_visible_child_name)(stack, name.as_ptr());
    }
}

unsafe fn refresh_start_page() {
    let b = match BINDINGS.get() {
        Some(b) => b,
        None => return,
    };

    let (recent_items, bookmark_items, breadcrumbs, search_placeholder) = {
        let db_guard = BROWSER_DB.lock().ok();
        let prof_guard = ACTIVE_PROFILE_ID.lock().ok();
        let mut model_guard = START_PAGE_MODEL.lock().ok();

        let db = db_guard.as_ref().and_then(|g| g.as_ref());
        let prof_id = prof_guard.as_ref().and_then(|g| g.as_ref()).copied();
        let model = model_guard.as_mut().and_then(|g| g.as_mut());

        if let (Some(db), Some(profile_id), Some(model)) = (db, prof_id, model) {
            let _ = model.reload(db, profile_id);
            (
                model.recent_items.clone(),
                model.bookmark_items.clone(),
                model.breadcrumbs.clone(),
                model.search_placeholder.clone(),
            )
        } else {
            return;
        }
    };

    // Update search placeholder
    let search_entry = ACTIVE_START_PAGE_SEARCH.load(Ordering::SeqCst);
    if !search_entry.is_null() {
        let c_place = CString::new(search_placeholder).unwrap();
        (b.gtk_entry_set_placeholder_text)(search_entry, c_place.as_ptr());
    }

    // Refresh Recent items
    let recent_flow = ACTIVE_RECENT_FLOW.load(Ordering::SeqCst);
    let recent_box = ACTIVE_RECENT_BOX.load(Ordering::SeqCst);
    if !recent_flow.is_null() && !recent_box.is_null() {
        loop {
            let child = (b.gtk_widget_get_first_child)(recent_flow);
            if child.is_null() {
                break;
            }
            (b.gtk_flow_box_remove)(recent_flow, child);
        }

        if recent_items.is_empty() {
            (b.gtk_widget_set_visible)(recent_box, 0);
        } else {
            (b.gtk_widget_set_visible)(recent_box, 1);
            let clicked_sig = CString::new("clicked").unwrap();
            let cb_card_click = std::mem::transmute::<
                unsafe extern "C" fn(*mut c_void, *mut c_void),
                GCallback,
            >(on_recent_card_clicked);

            for (idx, item) in recent_items.iter().enumerate() {
                let card_btn = (b.gtk_button_new)();
                (b.gtk_button_set_has_frame)(card_btn, 0);
                (b.gtk_widget_add_css_class)(
                    card_btn,
                    CString::new("recent-card").unwrap().as_ptr(),
                );
                (b.gtk_widget_set_size_request)(card_btn, 175, 120);

                let card_box = (b.gtk_box_new)(1, 4); // Vertical
                (b.gtk_widget_set_margin_start)(card_box, 6);
                (b.gtk_widget_set_margin_end)(card_box, 6);
                (b.gtk_widget_set_margin_top)(card_box, 6);
                (b.gtk_widget_set_margin_bottom)(card_box, 6);

                let preview_widget = if item.has_preview {
                    if let Some(ref ppath) = item.preview_path {
                        if let Ok(c_path) = CString::new(ppath.clone()) {
                            (b.gtk_picture_new_for_filename)(c_path.as_ptr())
                        } else {
                            (b.gtk_image_new_from_icon_name)(
                                CString::new("globe-symbolic").unwrap().as_ptr(),
                            )
                        }
                    } else {
                        (b.gtk_image_new_from_icon_name)(
                            CString::new("globe-symbolic").unwrap().as_ptr(),
                        )
                    }
                } else {
                    let fallback_img = (b.gtk_image_new_from_icon_name)(
                        CString::new("globe-symbolic").unwrap().as_ptr(),
                    );
                    (b.gtk_image_set_pixel_size)(fallback_img, 36);
                    fallback_img
                };
                (b.gtk_widget_set_size_request)(preview_widget, 160, 72);
                (b.gtk_widget_add_css_class)(
                    preview_widget,
                    CString::new("recent-preview").unwrap().as_ptr(),
                );

                let lbl_title =
                    (b.gtk_label_new)(CString::new(item.title.clone()).unwrap().as_ptr());
                (b.gtk_label_set_xalign)(lbl_title, 0.0);
                (b.gtk_label_set_ellipsize)(lbl_title, 3); // PANGO_ELLIPSIZE_END = 3
                (b.gtk_widget_add_css_class)(
                    lbl_title,
                    CString::new("recent-card-title").unwrap().as_ptr(),
                );

                let lbl_domain =
                    (b.gtk_label_new)(CString::new(item.domain.clone()).unwrap().as_ptr());
                (b.gtk_label_set_xalign)(lbl_domain, 0.0);
                (b.gtk_label_set_ellipsize)(lbl_domain, 3);
                (b.gtk_widget_add_css_class)(
                    lbl_domain,
                    CString::new("recent-card-domain").unwrap().as_ptr(),
                );

                (b.gtk_box_append)(card_box, preview_widget);
                (b.gtk_box_append)(card_box, lbl_title);
                (b.gtk_box_append)(card_box, lbl_domain);
                (b.gtk_button_set_child)(card_btn, card_box);

                (b.gtk_widget_set_tooltip_text)(
                    card_btn,
                    CString::new(item.url.clone()).unwrap().as_ptr(),
                );

                (b.g_signal_connect_data)(
                    card_btn,
                    clicked_sig.as_ptr(),
                    cb_card_click,
                    idx as *mut c_void,
                    std::ptr::null_mut(),
                    0,
                );

                (b.gtk_flow_box_append)(recent_flow, card_btn);
            }
        }
    }

    // Refresh Breadcrumbs
    let breadcrumbs_box = ACTIVE_BREADCRUMBS_BOX.load(Ordering::SeqCst);
    if !breadcrumbs_box.is_null() {
        loop {
            let child = (b.gtk_widget_get_first_child)(breadcrumbs_box);
            if child.is_null() {
                break;
            }
            (b.gtk_box_remove)(breadcrumbs_box, child);
        }

        if let Ok(mut list) = VISIBLE_BREADCRUMB_BTNS.lock() {
            list.clear();
        }

        let clicked_sig = CString::new("clicked").unwrap();
        let cb_crumb_click = std::mem::transmute::<
            unsafe extern "C" fn(*mut c_void, *mut c_void),
            GCallback,
        >(on_breadcrumb_clicked);

        let total_crumbs = breadcrumbs.len();
        for (idx, crumb) in breadcrumbs.iter().enumerate() {
            let btn_crumb =
                (b.gtk_button_new_with_label)(CString::new(crumb.title.clone()).unwrap().as_ptr());
            (b.gtk_button_set_has_frame)(btn_crumb, 0);
            (b.gtk_widget_add_css_class)(
                btn_crumb,
                CString::new("breadcrumb-btn").unwrap().as_ptr(),
            );

            if let Ok(mut list) = VISIBLE_BREADCRUMB_BTNS.lock() {
                list.push((crumb.id, btn_crumb as usize));
            }

            (b.g_signal_connect_data)(
                btn_crumb,
                clicked_sig.as_ptr(),
                cb_crumb_click,
                idx as *mut c_void,
                std::ptr::null_mut(),
                0,
            );
            (b.gtk_box_append)(breadcrumbs_box, btn_crumb);

            if idx + 1 < total_crumbs {
                let lbl_sep = (b.gtk_label_new)(CString::new("›").unwrap().as_ptr());
                (b.gtk_widget_add_css_class)(
                    lbl_sep,
                    CString::new("breadcrumb-sep").unwrap().as_ptr(),
                );
                (b.gtk_box_append)(breadcrumbs_box, lbl_sep);
            }
        }
    }

    // Refresh Bookmarks flowbox
    let bookmarks_flow = ACTIVE_BOOKMARKS_FLOW.load(Ordering::SeqCst);
    if !bookmarks_flow.is_null() {
        loop {
            let child = (b.gtk_widget_get_first_child)(bookmarks_flow);
            if child.is_null() {
                break;
            }
            (b.gtk_flow_box_remove)(bookmarks_flow, child);
        }

        if let Ok(mut list) = VISIBLE_FOLDER_TILES.lock() {
            list.clear();
        }

        let clicked_sig = CString::new("clicked").unwrap();
        let cb_tile_click = std::mem::transmute::<
            unsafe extern "C" fn(*mut c_void, *mut c_void),
            GCallback,
        >(on_bookmark_tile_clicked);

        let cb_drag_begin = std::mem::transmute::<
            unsafe extern "C" fn(*mut c_void, libc::c_double, libc::c_double, *mut c_void),
            GCallback,
        >(on_bookmark_drag_begin);
        let cb_drag_end = std::mem::transmute::<
            unsafe extern "C" fn(*mut c_void, libc::c_double, libc::c_double, *mut c_void),
            GCallback,
        >(on_bookmark_drag_end);

        let drag_begin_sig = CString::new("drag-begin").unwrap();
        let drag_end_sig = CString::new("drag-end").unwrap();

        for (idx, item) in bookmark_items.iter().enumerate() {
            let tile_btn = (b.gtk_button_new)();
            (b.gtk_button_set_has_frame)(tile_btn, 0);
            (b.gtk_widget_add_css_class)(tile_btn, CString::new("bookmark-tile").unwrap().as_ptr());
            (b.gtk_widget_set_size_request)(tile_btn, 130, 96);

            let tile_box = (b.gtk_box_new)(1, 4); // Vertical
            (b.gtk_widget_set_halign)(tile_box, 3); // GTK_ALIGN_CENTER = 3
            (b.gtk_widget_set_valign)(tile_box, 3);

            let icon_name = if item.is_folder {
                "folder-symbolic"
            } else {
                "user-bookmarks-symbolic"
            };
            let icon_img =
                (b.gtk_image_new_from_icon_name)(CString::new(icon_name).unwrap().as_ptr());
            (b.gtk_image_set_pixel_size)(icon_img, 32);

            let lbl_title = (b.gtk_label_new)(CString::new(item.title.clone()).unwrap().as_ptr());
            (b.gtk_label_set_xalign)(lbl_title, 0.5);
            (b.gtk_label_set_ellipsize)(lbl_title, 3);
            (b.gtk_widget_add_css_class)(
                lbl_title,
                CString::new("bookmark-tile-title").unwrap().as_ptr(),
            );

            (b.gtk_box_append)(tile_box, icon_img);
            (b.gtk_box_append)(tile_box, lbl_title);
            (b.gtk_button_set_child)(tile_btn, tile_box);

            if let Some(ref u) = item.url {
                (b.gtk_widget_set_tooltip_text)(
                    tile_btn,
                    CString::new(u.clone()).unwrap().as_ptr(),
                );
            }

            if item.is_folder {
                if let Ok(mut list) = VISIBLE_FOLDER_TILES.lock() {
                    list.push((item.id, tile_btn as usize));
                }
            }

            // Drag gesture controller
            let drag_gesture = (b.gtk_gesture_drag_new)();
            (b.gtk_widget_add_controller)(tile_btn, drag_gesture);

            (b.g_signal_connect_data)(
                drag_gesture,
                drag_begin_sig.as_ptr(),
                cb_drag_begin,
                idx as *mut c_void,
                std::ptr::null_mut(),
                0,
            );
            (b.g_signal_connect_data)(
                drag_gesture,
                drag_end_sig.as_ptr(),
                cb_drag_end,
                idx as *mut c_void,
                std::ptr::null_mut(),
                0,
            );

            // Click signal
            (b.g_signal_connect_data)(
                tile_btn,
                clicked_sig.as_ptr(),
                cb_tile_click,
                idx as *mut c_void,
                std::ptr::null_mut(),
                0,
            );

            (b.gtk_flow_box_append)(bookmarks_flow, tile_btn);
        }
    }
}

static REFRESH_PENDING: AtomicBool = AtomicBool::new(false);

unsafe extern "C" fn on_idle_refresh(_user_data: *mut c_void) -> libc::c_int {
    REFRESH_PENDING.store(false, Ordering::SeqCst);
    refresh_start_page();
    0 // G_SOURCE_REMOVE
}

unsafe fn queue_refresh_start_page() {
    if let Some(b) = BINDINGS.get() {
        if !REFRESH_PENDING.swap(true, Ordering::SeqCst) {
            (b.g_idle_add)(on_idle_refresh, std::ptr::null_mut());
        }
    } else {
        refresh_start_page();
    }
}

unsafe extern "C" fn on_start_page_search_activate(entry: *mut c_void, _user_data: *mut c_void) {
    if entry.is_null() {
        return;
    }
    let b = match BINDINGS.get() {
        Some(b) => b,
        None => return,
    };
    let text_ptr = (b.gtk_editable_get_text)(entry);
    if text_ptr.is_null() {
        return;
    }
    let query = CStr::from_ptr(text_ptr)
        .to_string_lossy()
        .trim()
        .to_string();
    if query.is_empty() {
        return;
    }

    let url_opt = {
        let model_guard = START_PAGE_MODEL.lock().ok();
        if let Some(Some(model)) = model_guard.as_deref() {
            model.resolve_search_input(&query)
        } else {
            None
        }
    };

    if let Some(target_url) = url_opt {
        navigate_to_url(&target_url);
    }
}

unsafe extern "C" fn on_recent_card_clicked(_button: *mut c_void, user_data: *mut c_void) {
    let idx = user_data as usize;
    let url_opt = {
        let model_guard = START_PAGE_MODEL.lock().ok();
        model_guard
            .as_deref()
            .and_then(|g| g.as_ref())
            .and_then(|m| m.recent_items.get(idx))
            .map(|item| item.url.clone())
    };

    if let Some(target_url) = url_opt {
        navigate_to_url(&target_url);
    }
}

unsafe extern "C" fn on_breadcrumb_clicked(_button: *mut c_void, user_data: *mut c_void) {
    let idx = user_data as usize;
    let target_folder_id = {
        let model_guard = START_PAGE_MODEL.lock().ok();
        model_guard
            .as_deref()
            .and_then(|g| g.as_ref())
            .and_then(|m| m.breadcrumbs.get(idx))
            .map(|crumb| crumb.id)
    };

    if let Some(folder_id) = target_folder_id {
        {
            let db_guard = BROWSER_DB.lock().ok();
            let prof_guard = ACTIVE_PROFILE_ID.lock().ok();
            let mut model_guard = START_PAGE_MODEL.lock().ok();

            let db = db_guard.as_ref().and_then(|g| g.as_ref());
            let prof_id = prof_guard.as_ref().and_then(|g| g.as_ref()).copied();
            let model = model_guard.as_mut().and_then(|g| g.as_mut());

            if let (Some(db), Some(profile_id), Some(model)) = (db, prof_id, model) {
                let _ = model.navigate_folder(db, profile_id, folder_id);
            }
        }
        queue_refresh_start_page();
    }
}

unsafe extern "C" fn on_bookmark_tile_clicked(_button: *mut c_void, user_data: *mut c_void) {
    let idx = user_data as usize;
    let item_opt = {
        let model_guard = START_PAGE_MODEL.lock().ok();
        model_guard
            .as_deref()
            .and_then(|g| g.as_ref())
            .and_then(|m| m.bookmark_items.get(idx))
            .cloned()
    };

    if let Some(item) = item_opt {
        if item.is_folder {
            {
                let db_guard = BROWSER_DB.lock().ok();
                let prof_guard = ACTIVE_PROFILE_ID.lock().ok();
                let mut model_guard = START_PAGE_MODEL.lock().ok();

                let db = db_guard.as_ref().and_then(|g| g.as_ref());
                let prof_id = prof_guard.as_ref().and_then(|g| g.as_ref()).copied();
                let model = model_guard.as_mut().and_then(|g| g.as_mut());

                if let (Some(db), Some(profile_id), Some(model)) = (db, prof_id, model) {
                    let _ = model.navigate_folder(db, profile_id, Some(item.id));
                }
            }
            queue_refresh_start_page();
        } else if let Some(ref url) = item.url {
            navigate_to_url(url);
        }
    }
}

unsafe extern "C" fn on_new_folder_clicked(_button: *mut c_void, _user_data: *mut c_void) {
    {
        let db_guard = BROWSER_DB.lock().ok();
        let prof_guard = ACTIVE_PROFILE_ID.lock().ok();
        let mut model_guard = START_PAGE_MODEL.lock().ok();

        let db = db_guard.as_ref().and_then(|g| g.as_ref());
        let prof_id = prof_guard.as_ref().and_then(|g| g.as_ref()).copied();
        let model = model_guard.as_mut().and_then(|g| g.as_mut());

        if let (Some(db), Some(profile_id), Some(model)) = (db, prof_id, model) {
            let folder_count = model.bookmark_items.iter().filter(|b| b.is_folder).count() + 1;
            let title = format!("New Folder {}", folder_count);
            let _ = model.create_folder(db, profile_id, &title);
        }
    }
    queue_refresh_start_page();
}

unsafe extern "C" fn on_add_bookmark_clicked(_button: *mut c_void, _user_data: *mut c_void) {
    {
        let db_guard = BROWSER_DB.lock().ok();
        let prof_guard = ACTIVE_PROFILE_ID.lock().ok();
        let mut model_guard = START_PAGE_MODEL.lock().ok();

        let db = db_guard.as_ref().and_then(|g| g.as_ref());
        let prof_id = prof_guard.as_ref().and_then(|g| g.as_ref()).copied();
        let model = model_guard.as_mut().and_then(|g| g.as_mut());

        if let (Some(db), Some(profile_id), Some(model)) = (db, prof_id, model) {
            let count = model.bookmark_items.iter().filter(|b| !b.is_folder).count() + 1;
            let title = format!("Bookmark {}", count);
            let url = "https://duckduckgo.com";
            let _ = model.create_bookmark(db, profile_id, &title, url);
        }
    }
    queue_refresh_start_page();
}

unsafe extern "C" fn on_bookmark_drag_begin(
    _gesture: *mut c_void,
    _start_x: libc::c_double,
    _start_y: libc::c_double,
    user_data: *mut c_void,
) {
    let idx = user_data as usize;
    let item_id = {
        let model_guard = START_PAGE_MODEL.lock().ok();
        model_guard
            .as_deref()
            .and_then(|g| g.as_ref())
            .and_then(|m| m.bookmark_items.get(idx))
            .map(|item| item.id)
    };

    if let Ok(mut dragged_guard) = DRAGGED_ITEM_ID.lock() {
        *dragged_guard = item_id;
    }
}

unsafe extern "C" fn on_bookmark_drag_end(
    gesture: *mut c_void,
    offset_x: libc::c_double,
    offset_y: libc::c_double,
    user_data: *mut c_void,
) {
    let dist_sq = offset_x * offset_x + offset_y * offset_y;
    if dist_sq < 100.0 {
        if let Ok(mut dragged_guard) = DRAGGED_ITEM_ID.lock() {
            *dragged_guard = None;
        }
        return;
    }

    let b = match BINDINGS.get() {
        Some(b) => b,
        None => return,
    };

    let dragged_id = match DRAGGED_ITEM_ID.lock().ok().and_then(|mut g| g.take()) {
        Some(id) => id,
        None => {
            let idx = user_data as usize;
            let model_guard = START_PAGE_MODEL.lock().ok();
            match model_guard
                .as_deref()
                .and_then(|g| g.as_ref())
                .and_then(|m| m.bookmark_items.get(idx))
                .map(|item| item.id)
            {
                Some(id) => id,
                None => return,
            }
        }
    };

    let widget = (b.gtk_event_controller_get_widget)(gesture);
    if widget.is_null() {
        return;
    }

    let mut start_x = 0.0;
    let mut start_y = 0.0;
    (b.gtk_gesture_drag_get_start_point)(gesture, &mut start_x, &mut start_y);
    let drop_x = start_x + offset_x;
    let drop_y = start_y + offset_y;

    // Check if dropped onto a visible breadcrumb button
    let mut target_folder_id = None;
    let mut dropped_on_target = false;

    if let Ok(crumbs) = VISIBLE_BREADCRUMB_BTNS.lock() {
        for (crumb_folder_id, btn_ptr) in crumbs.iter() {
            let target_widget = *btn_ptr as *mut c_void;
            let mut rel_x = 0.0;
            let mut rel_y = 0.0;
            let ok = (b.gtk_widget_translate_coordinates)(
                widget,
                target_widget,
                drop_x,
                drop_y,
                &mut rel_x,
                &mut rel_y,
            );
            if ok != 0 {
                let w = (b.gtk_widget_get_width)(target_widget) as f64;
                let h = (b.gtk_widget_get_height)(target_widget) as f64;
                if rel_x >= 0.0 && rel_x <= w && rel_y >= 0.0 && rel_y <= h {
                    target_folder_id = *crumb_folder_id;
                    dropped_on_target = true;
                    break;
                }
            }
        }
    }

    // Check if dropped onto a visible folder tile
    if !dropped_on_target {
        if let Ok(folders) = VISIBLE_FOLDER_TILES.lock() {
            for (folder_id, tile_ptr) in folders.iter() {
                if *folder_id == dragged_id {
                    continue; // cannot drop inside itself
                }
                let target_widget = *tile_ptr as *mut c_void;
                let mut rel_x = 0.0;
                let mut rel_y = 0.0;
                let ok = (b.gtk_widget_translate_coordinates)(
                    widget,
                    target_widget,
                    drop_x,
                    drop_y,
                    &mut rel_x,
                    &mut rel_y,
                );
                if ok != 0 {
                    let w = (b.gtk_widget_get_width)(target_widget) as f64;
                    let h = (b.gtk_widget_get_height)(target_widget) as f64;
                    if rel_x >= 0.0 && rel_x <= w && rel_y >= 0.0 && rel_y <= h {
                        target_folder_id = Some(*folder_id);
                        dropped_on_target = true;
                        break;
                    }
                }
            }
        }
    }

    if dropped_on_target {
        {
            let db_guard = BROWSER_DB.lock().ok();
            let prof_guard = ACTIVE_PROFILE_ID.lock().ok();
            let mut model_guard = START_PAGE_MODEL.lock().ok();

            let db = db_guard.as_ref().and_then(|g| g.as_ref());
            let prof_id = prof_guard.as_ref().and_then(|g| g.as_ref()).copied();
            let model = model_guard.as_mut().and_then(|g| g.as_mut());

            if let (Some(db), Some(profile_id), Some(model)) = (db, prof_id, model) {
                let _ = model.move_item(db, profile_id, dragged_id, target_folder_id);
            }
        }
        queue_refresh_start_page();
    }
}

const GNOME_ACCENTS: &[(&str, &str, &str)] = &[
    ("Default", "#808080", "none"),
    ("Blue", "#3584e4", "radial-gradient(ellipse at 50% -10%, rgba(53, 132, 228, 0.35) 0%, rgba(53, 132, 228, 0.10) 45%, transparent 75%)"),
    ("Teal", "#2190a4", "radial-gradient(ellipse at 50% -10%, rgba(33, 144, 164, 0.35) 0%, rgba(33, 144, 164, 0.10) 45%, transparent 75%)"),
    ("Green", "#3a944a", "radial-gradient(ellipse at 50% -10%, rgba(58, 148, 74, 0.35) 0%, rgba(58, 148, 74, 0.10) 45%, transparent 75%)"),
    ("Yellow", "#c88800", "radial-gradient(ellipse at 50% -10%, rgba(200, 136, 0, 0.35) 0%, rgba(200, 136, 0, 0.10) 45%, transparent 75%)"),
    ("Orange", "#ed5b00", "radial-gradient(ellipse at 50% -10%, rgba(237, 91, 0, 0.35) 0%, rgba(237, 91, 0, 0.10) 45%, transparent 75%)"),
    ("Red", "#e62d42", "radial-gradient(ellipse at 50% -10%, rgba(230, 45, 66, 0.35) 0%, rgba(230, 45, 66, 0.10) 45%, transparent 75%)"),
    ("Pink", "#d56199", "radial-gradient(ellipse at 50% -10%, rgba(213, 97, 153, 0.35) 0%, rgba(213, 97, 153, 0.10) 45%, transparent 75%)"),
    ("Purple", "#9141ac", "radial-gradient(ellipse at 50% -10%, rgba(145, 65, 172, 0.35) 0%, rgba(145, 65, 172, 0.10) 45%, transparent 75%)"),
    ("Slate", "#6f8396", "radial-gradient(ellipse at 50% -10%, rgba(111, 131, 150, 0.35) 0%, rgba(111, 131, 150, 0.10) 45%, transparent 75%)"),
];

unsafe fn apply_start_page_accent(name: &str) {
    let b = match BINDINGS.get() {
        Some(b) => b,
        None => return,
    };
    let bg = GNOME_ACCENTS
        .iter()
        .find(|(n, _, _)| n.eq_ignore_ascii_case(name))
        .map(|(_, _, bg)| *bg)
        .unwrap_or("none");

    let css = format!(
        ".start-page-container {{ background: {}; }}\n.start-page-scrolled {{ background: transparent; }}",
        bg
    );

    let display = (b.gdk_display_get_default)();
    if !display.is_null() {
        let mut provider = ACCENT_CSS_PROVIDER.load(Ordering::SeqCst);
        if provider.is_null() {
            provider = (b.gtk_css_provider_new)();
            ACCENT_CSS_PROVIDER.store(provider, Ordering::SeqCst);
            (b.gtk_style_context_add_provider_for_display)(display, provider, 800);
        }
        if let Ok(c_css) = CString::new(css) {
            (b.gtk_css_provider_load_from_string)(provider, c_css.as_ptr());
        }
    }

    let path = default_data_dir().join("start_page_accent.txt");
    let _ = std::fs::write(path, name);
}

fn load_saved_accent() -> String {
    let path = default_data_dir().join("start_page_accent.txt");
    std::fs::read_to_string(path)
        .unwrap_or_else(|_| "Blue".to_string())
        .trim()
        .to_string()
}

unsafe extern "C" fn on_accent_button_clicked(_button: *mut c_void, user_data: *mut c_void) {
    let idx = user_data as usize;
    if let Some((name, _, _)) = GNOME_ACCENTS.get(idx) {
        apply_start_page_accent(name);
    }
}

unsafe fn build_start_page_widget(b: &GtkBindings) -> *mut c_void {
    let scrolled = (b.gtk_scrolled_window_new)();
    (b.gtk_scrolled_window_set_policy)(scrolled, 2, 1); // H=Never (2), V=Automatic (1)
    (b.gtk_widget_set_vexpand)(scrolled, 1);
    (b.gtk_widget_set_hexpand)(scrolled, 1);
    (b.gtk_widget_add_css_class)(
        scrolled,
        CString::new("start-page-scrolled").unwrap().as_ptr(),
    );

    let container = (b.gtk_box_new)(1, 24); // Vertical, 24px spacing
    (b.gtk_widget_add_css_class)(
        container,
        CString::new("start-page-container").unwrap().as_ptr(),
    );
    (b.gtk_widget_set_margin_start)(container, 32);
    (b.gtk_widget_set_margin_end)(container, 32);
    (b.gtk_widget_set_margin_top)(container, 40);
    (b.gtk_widget_set_margin_bottom)(container, 48);

    // Section 1: Hero & Search
    let hero_box = (b.gtk_box_new)(1, 14);
    (b.gtk_widget_set_halign)(hero_box, 3); // GTK_ALIGN_CENTER

    let logo_lbl = (b.gtk_label_new)(CString::new("Vingilot").unwrap().as_ptr());
    (b.gtk_widget_add_css_class)(logo_lbl, CString::new("start-page-logo").unwrap().as_ptr());
    (b.gtk_box_append)(hero_box, logo_lbl);

    let search_entry = (b.gtk_entry_new)();
    ACTIVE_START_PAGE_SEARCH.store(search_entry, Ordering::SeqCst);
    (b.gtk_widget_add_css_class)(
        search_entry,
        CString::new("start-page-search").unwrap().as_ptr(),
    );
    (b.gtk_widget_set_size_request)(search_entry, 520, 44);

    let activate_sig = CString::new("activate").unwrap();
    let cb_search_activate = std::mem::transmute::<
        unsafe extern "C" fn(*mut c_void, *mut c_void),
        GCallback,
    >(on_start_page_search_activate);

    (b.g_signal_connect_data)(
        search_entry,
        activate_sig.as_ptr(),
        cb_search_activate,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
    );

    (b.gtk_box_append)(hero_box, search_entry);

    // Accent Palette Selector (Standard GNOME Accents)
    let palette_box = (b.gtk_box_new)(0, 6);
    (b.gtk_widget_set_halign)(palette_box, 3); // GTK_ALIGN_CENTER
    (b.gtk_widget_add_css_class)(
        palette_box,
        CString::new("accent-palette-box").unwrap().as_ptr(),
    );

    let lbl_palette = (b.gtk_label_new)(CString::new("🎨 Accent:").unwrap().as_ptr());
    (b.gtk_widget_add_css_class)(
        lbl_palette,
        CString::new("accent-palette-label").unwrap().as_ptr(),
    );
    (b.gtk_box_append)(palette_box, lbl_palette);

    let accents_classes: &[(&str, &str)] = &[
        ("Default", "accent-dot-default"),
        ("Blue", "accent-dot-blue"),
        ("Teal", "accent-dot-teal"),
        ("Green", "accent-dot-green"),
        ("Yellow", "accent-dot-yellow"),
        ("Orange", "accent-dot-orange"),
        ("Red", "accent-dot-red"),
        ("Pink", "accent-dot-pink"),
        ("Purple", "accent-dot-purple"),
        ("Slate", "accent-dot-slate"),
    ];

    let clicked_sig = CString::new("clicked").unwrap();
    let cb_accent = std::mem::transmute::<unsafe extern "C" fn(*mut c_void, *mut c_void), GCallback>(
        on_accent_button_clicked,
    );

    for (idx, (acc_name, dot_class)) in accents_classes.iter().enumerate() {
        let dot_btn = (b.gtk_button_new)();
        (b.gtk_button_set_has_frame)(dot_btn, 0);
        (b.gtk_widget_set_size_request)(dot_btn, 22, 22);
        (b.gtk_widget_add_css_class)(dot_btn, CString::new("accent-dot-btn").unwrap().as_ptr());
        (b.gtk_widget_add_css_class)(dot_btn, CString::new(*dot_class).unwrap().as_ptr());
        (b.gtk_widget_set_tooltip_text)(
            dot_btn,
            CString::new(format!("{} accent", acc_name))
                .unwrap()
                .as_ptr(),
        );

        (b.g_signal_connect_data)(
            dot_btn,
            clicked_sig.as_ptr(),
            cb_accent,
            idx as *mut c_void,
            std::ptr::null_mut(),
            0,
        );
        (b.gtk_box_append)(palette_box, dot_btn);
    }
    (b.gtk_box_append)(hero_box, palette_box);

    (b.gtk_box_append)(container, hero_box);

    // Section 2: Recently viewed
    let recent_box = (b.gtk_box_new)(1, 8);
    ACTIVE_RECENT_BOX.store(recent_box, Ordering::SeqCst);
    (b.gtk_widget_add_css_class)(
        recent_box,
        CString::new("start-page-section").unwrap().as_ptr(),
    );

    let recent_header = (b.gtk_box_new)(0, 6);
    let lbl_recent = (b.gtk_label_new)(CString::new("Recently Viewed").unwrap().as_ptr());
    (b.gtk_label_set_xalign)(lbl_recent, 0.0);
    (b.gtk_widget_add_css_class)(
        lbl_recent,
        CString::new("section-header-title").unwrap().as_ptr(),
    );
    (b.gtk_box_append)(recent_header, lbl_recent);
    (b.gtk_box_append)(recent_box, recent_header);

    let recent_flow = (b.gtk_flow_box_new)();
    ACTIVE_RECENT_FLOW.store(recent_flow, Ordering::SeqCst);
    (b.gtk_flow_box_set_selection_mode)(recent_flow, 0); // GTK_SELECTION_NONE = 0
    (b.gtk_flow_box_set_min_children_per_line)(recent_flow, 1);
    (b.gtk_flow_box_set_max_children_per_line)(recent_flow, 5);
    (b.gtk_flow_box_set_column_spacing)(recent_flow, 14);
    (b.gtk_flow_box_set_row_spacing)(recent_flow, 14);
    (b.gtk_flow_box_set_homogeneous)(recent_flow, 1);
    (b.gtk_box_append)(recent_box, recent_flow);
    (b.gtk_box_append)(container, recent_box);

    // Section 3: Bookmarks Toolbar and FlowBox
    let bm_section = (b.gtk_box_new)(1, 8);
    (b.gtk_widget_add_css_class)(
        bm_section,
        CString::new("start-page-section").unwrap().as_ptr(),
    );

    let bm_toolbar = (b.gtk_box_new)(0, 8); // Horizontal toolbar
    let breadcrumbs_box = (b.gtk_box_new)(0, 4);
    ACTIVE_BREADCRUMBS_BOX.store(breadcrumbs_box, Ordering::SeqCst);
    (b.gtk_widget_set_hexpand)(breadcrumbs_box, 1);
    (b.gtk_box_append)(bm_toolbar, breadcrumbs_box);

    let clicked_sig = CString::new("clicked").unwrap();
    let btn_new_folder =
        (b.gtk_button_new_with_label)(CString::new("📁 New Folder").unwrap().as_ptr());
    (b.gtk_widget_add_css_class)(
        btn_new_folder,
        CString::new("toolbar-btn").unwrap().as_ptr(),
    );
    let cb_new_folder = std::mem::transmute::<
        unsafe extern "C" fn(*mut c_void, *mut c_void),
        GCallback,
    >(on_new_folder_clicked);
    (b.g_signal_connect_data)(
        btn_new_folder,
        clicked_sig.as_ptr(),
        cb_new_folder,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
    );

    let btn_add_bm =
        (b.gtk_button_new_with_label)(CString::new("★ Add Bookmark").unwrap().as_ptr());
    (b.gtk_widget_add_css_class)(btn_add_bm, CString::new("toolbar-btn").unwrap().as_ptr());
    let cb_add_bm = std::mem::transmute::<unsafe extern "C" fn(*mut c_void, *mut c_void), GCallback>(
        on_add_bookmark_clicked,
    );
    (b.g_signal_connect_data)(
        btn_add_bm,
        clicked_sig.as_ptr(),
        cb_add_bm,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
    );

    (b.gtk_box_append)(bm_toolbar, btn_new_folder);
    (b.gtk_box_append)(bm_toolbar, btn_add_bm);
    (b.gtk_box_append)(bm_section, bm_toolbar);

    let bookmarks_flow = (b.gtk_flow_box_new)();
    ACTIVE_BOOKMARKS_FLOW.store(bookmarks_flow, Ordering::SeqCst);
    (b.gtk_flow_box_set_selection_mode)(bookmarks_flow, 0); // GTK_SELECTION_NONE
    (b.gtk_flow_box_set_min_children_per_line)(bookmarks_flow, 2);
    (b.gtk_flow_box_set_max_children_per_line)(bookmarks_flow, 12);
    (b.gtk_flow_box_set_column_spacing)(bookmarks_flow, 14);
    (b.gtk_flow_box_set_row_spacing)(bookmarks_flow, 14);
    (b.gtk_flow_box_set_homogeneous)(bookmarks_flow, 1);
    (b.gtk_box_append)(bm_section, bookmarks_flow);

    (b.gtk_box_append)(container, bm_section);

    (b.gtk_scrolled_window_set_child)(scrolled, container);
    scrolled
}

unsafe extern "C" fn on_home_clicked(_button: *mut c_void, _user_data: *mut c_void) {
    if let Ok(mut mgr_guard) = TAB_MANAGER.lock() {
        if let Some(mgr) = mgr_guard.as_mut() {
            if let Some(active) = mgr.active_tab_mut() {
                active.is_start_page = true;
                active.url = "about:blank".to_string();
                active.title = "Start Page".to_string();
            }
        }
    }
    switch_to_start_page();
    refresh_sidebar_tabs();
}

unsafe fn make_group_card(b: &GtkBindings, title: &str) -> *mut c_void {
    let card = (b.gtk_box_new)(1, 6);
    let card_class = CString::new("menu-group-card").unwrap();
    (b.gtk_widget_add_css_class)(card, card_class.as_ptr());

    let lbl_title = (b.gtk_label_new)(CString::new(title).unwrap().as_ptr());
    (b.gtk_label_set_xalign)(lbl_title, 0.0);
    let title_class = CString::new("menu-group-title").unwrap();
    (b.gtk_widget_add_css_class)(lbl_title, title_class.as_ptr());
    (b.gtk_box_append)(card, lbl_title);

    card
}

unsafe extern "C" fn on_app_activate(app: *mut c_void, _user_data: *mut c_void) {
    println!("[Vingilot] GApplication activate signal received - initializing Adwaita window");
    let b = match BINDINGS.get() {
        Some(bindings) => bindings,
        None => return,
    };

    let win = (b.adw_application_window_new)(app);
    let is_incognito = IS_INCOGNITO.load(Ordering::SeqCst);

    let title = if is_incognito {
        CString::new("Vingilot — Incognito").unwrap()
    } else {
        CString::new("Vingilot").unwrap()
    };
    (b.gtk_window_set_title)(win, title.as_ptr());
    (b.gtk_window_set_default_size)(win, 1100, 750);

    // Apply unified chrome CSS styling for seamless headerbar and sidebar integration
    let display = (b.gdk_display_get_default)();
    if !display.is_null() {
        let provider = (b.gtk_css_provider_new)();
        let css = crate::theme::base_css();
        if let Ok(c_css) = CString::new(css) {
            (b.gtk_css_provider_load_from_string)(provider, c_css.as_ptr());
            (b.gtk_style_context_add_provider_for_display)(
                display, provider, 600, // GTK_STYLE_PROVIDER_PRIORITY_APPLICATION
            );
        }

        if is_incognito {
            let incog_provider = (b.gtk_css_provider_new)();
            let incog_css = crate::theme::incognito_css();
            if let Ok(c_incog) = CString::new(incog_css) {
                (b.gtk_css_provider_load_from_string)(incog_provider, c_incog.as_ptr());
                (b.gtk_style_context_add_provider_for_display)(
                    display,
                    incog_provider,
                    800, // GTK_STYLE_PROVIDER_PRIORITY_USER
                );
            }
        }
    }

    if is_incognito {
        (b.gtk_widget_add_css_class)(win, CString::new("incognito").unwrap().as_ptr());
    }

    // Header bar
    let header = (b.adw_header_bar_new)();

    let (btn_sidebar, _header_sep, btn_back, btn_fwd, btn_refresh) = if is_incognito {
        // In incognito mode, suppress window title buttons at start if any
        (b.adw_header_bar_set_show_start_title_buttons)(header, 0);

        // Visible incognito icon on the corner left instead of the three buttons (#ffc95c)
        let btn_incognito_icon = (b.gtk_button_new)();
        let icon_img = (b.gtk_image_new_from_icon_name)(
            CString::new("vingilot-incognito-symbolic")
                .unwrap()
                .as_ptr(),
        );
        (b.gtk_image_set_pixel_size)(icon_img, 22);
        (b.gtk_button_set_child)(btn_incognito_icon, icon_img);
        (b.gtk_button_set_has_frame)(btn_incognito_icon, 0);
        (b.gtk_widget_set_size_request)(btn_incognito_icon, 38, 38);
        (b.gtk_widget_set_margin_start)(btn_incognito_icon, 8);
        (b.gtk_widget_set_margin_end)(btn_incognito_icon, 6);
        let incognito_tip = CString::new("Incognito Mode").unwrap();
        (b.gtk_widget_set_tooltip_text)(btn_incognito_icon, incognito_tip.as_ptr());
        let class_incog = CString::new("incognito-corner-icon").unwrap();
        (b.gtk_widget_add_css_class)(btn_incognito_icon, class_incog.as_ptr());
        (b.gtk_widget_add_css_class)(icon_img, class_incog.as_ptr());
        (b.adw_header_bar_pack_start)(header, btn_incognito_icon);

        (
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    } else {
        // Normal mode: Left corner button with browser icon (toggles Tab Sidebar)
        let btn_sidebar = (b.gtk_button_new)();
        let icon_img = (b.gtk_image_new_from_icon_name)(
            CString::new("io.github.antux912012.Vingilot-symbolic")
                .unwrap()
                .as_ptr(),
        );
        (b.gtk_image_set_pixel_size)(icon_img, 22);
        (b.gtk_button_set_child)(btn_sidebar, icon_img);
        (b.gtk_button_set_has_frame)(btn_sidebar, 0);
        let sidebar_tip = CString::new("Vingilot - Toggle Tab Sidebar").unwrap();
        (b.gtk_widget_set_tooltip_text)(btn_sidebar, sidebar_tip.as_ptr());
        (b.gtk_widget_set_size_request)(btn_sidebar, 38, 38);
        (b.gtk_widget_set_margin_start)(btn_sidebar, 5);
        (b.gtk_widget_set_margin_end)(btn_sidebar, 5);
        (b.adw_header_bar_pack_start)(header, btn_sidebar);

        // Vertical separator in headerbar aligned with the sidebar boundary
        let header_sep = (b.gtk_separator_new)(1); // 1 = GTK_ORIENTATION_VERTICAL
        (b.gtk_widget_set_margin_start)(header_sep, 2);
        (b.gtk_widget_set_margin_end)(header_sep, 4);
        (b.adw_header_bar_pack_start)(header, header_sep);

        // Navigation buttons
        let back_icon = CString::new("go-previous-symbolic").unwrap();
        let btn_back = (b.gtk_button_new_from_icon_name)(back_icon.as_ptr());

        let fwd_icon = CString::new("go-next-symbolic").unwrap();
        let btn_fwd = (b.gtk_button_new_from_icon_name)(fwd_icon.as_ptr());

        let ref_icon = CString::new("view-refresh-symbolic").unwrap();
        let btn_refresh = (b.gtk_button_new_from_icon_name)(ref_icon.as_ptr());

        (b.adw_header_bar_pack_start)(header, btn_back);
        (b.adw_header_bar_pack_start)(header, btn_fwd);
        (b.adw_header_bar_pack_start)(header, btn_refresh);

        (btn_sidebar, header_sep, btn_back, btn_fwd, btn_refresh)
    };

    // Address bar entry & Lens Zoom button in title container
    let title_box = (b.gtk_box_new)(0, 4); // 0 = GTK_ORIENTATION_HORIZONTAL, 4px spacing
    (b.gtk_widget_set_hexpand)(title_box, 1);

    // Lens zoom button on left of URL entry
    let lens_btn = (b.gtk_menu_button_new)();
    ACTIVE_LENS_BTN.store(lens_btn, Ordering::SeqCst);
    let lens_icon = CString::new("system-search-symbolic").unwrap();
    (b.gtk_menu_button_set_icon_name)(lens_btn, lens_icon.as_ptr());
    let lens_tip = CString::new("Page Zoom").unwrap();
    (b.gtk_widget_set_tooltip_text)(lens_btn, lens_tip.as_ptr());
    (b.gtk_menu_button_set_has_frame)(lens_btn, 0);

    // Lens Popover
    let lens_popover = (b.gtk_popover_new)();
    (b.gtk_menu_button_set_popover)(lens_btn, lens_popover);

    let lens_box = (b.gtk_box_new)(1, 6); // 1 = GTK_ORIENTATION_VERTICAL, 6px spacing
    (b.gtk_widget_set_margin_start)(lens_box, 10);
    (b.gtk_widget_set_margin_end)(lens_box, 10);
    (b.gtk_widget_set_margin_top)(lens_box, 8);
    (b.gtk_widget_set_margin_bottom)(lens_box, 8);
    (b.gtk_popover_set_child)(lens_popover, lens_box);

    let lens_ctrl_box = (b.gtk_box_new)(0, 6); // Horizontal
    let btn_lens_out = (b.gtk_button_new_with_label)(CString::new(" − ").unwrap().as_ptr());
    let lbl_lens_val = (b.gtk_label_new)(CString::new("100%").unwrap().as_ptr());
    ACTIVE_ZOOM_LABEL.store(lbl_lens_val, Ordering::SeqCst);
    (b.gtk_widget_set_margin_start)(lbl_lens_val, 6);
    (b.gtk_widget_set_margin_end)(lbl_lens_val, 6);
    let btn_lens_in = (b.gtk_button_new_with_label)(CString::new(" + ").unwrap().as_ptr());
    let btn_lens_reset = (b.gtk_button_new_with_label)(CString::new("↺ Reset").unwrap().as_ptr());

    (b.gtk_box_append)(lens_ctrl_box, btn_lens_out);
    (b.gtk_box_append)(lens_ctrl_box, lbl_lens_val);
    (b.gtk_box_append)(lens_ctrl_box, btn_lens_in);
    (b.gtk_box_append)(lens_ctrl_box, btn_lens_reset);
    (b.gtk_box_append)(lens_box, lens_ctrl_box);

    let entry = (b.gtk_entry_new)();
    ACTIVE_ENTRY.store(entry, Ordering::SeqCst);
    let placeholder = CString::new("Search or enter web address...").unwrap();
    (b.gtk_entry_set_placeholder_text)(entry, placeholder.as_ptr());
    (b.gtk_widget_set_hexpand)(entry, 1);

    // Initialize Browser Database & Start Page model
    init_browser_storage(is_incognito);

    let initial_requested_url = INITIAL_URL.lock().ok().and_then(|g| g.clone());

    let (start_url, is_initial_start_page) = match initial_requested_url {
        Some(u) => {
            if u == "about:blank" || u == "browser://start" {
                ("about:blank".to_string(), true)
            } else {
                (u, false)
            }
        }
        None => ("about:blank".to_string(), true),
    };

    if !is_initial_start_page {
        if let Ok(c_start_url) = CString::new(start_url.clone()) {
            (b.gtk_editable_set_text)(entry, c_start_url.as_ptr());
        }
    }

    // Reader Mode Button on right side of address bar
    let reader_btn = (b.gtk_button_new)();
    let reader_icon_img =
        (b.gtk_image_new_from_icon_name)(CString::new("view-paged-symbolic").unwrap().as_ptr());
    (b.gtk_button_set_child)(reader_btn, reader_icon_img);
    (b.gtk_button_set_has_frame)(reader_btn, 0);
    (b.gtk_widget_set_size_request)(reader_btn, 32, 32);
    let reader_tip = CString::new("Toggle Distraction-Free Reader Mode").unwrap();
    (b.gtk_widget_set_tooltip_text)(reader_btn, reader_tip.as_ptr());
    let class_action_btn = CString::new("address-bar-action-btn").unwrap();
    (b.gtk_widget_add_css_class)(reader_btn, class_action_btn.as_ptr());

    // Bookmark Star Button on right side of address bar
    let star_btn = (b.gtk_button_new)();
    ACTIVE_STAR_BTN.store(star_btn, Ordering::SeqCst);
    let star_icon_img =
        (b.gtk_image_new_from_icon_name)(CString::new("non-starred-symbolic").unwrap().as_ptr());
    (b.gtk_button_set_child)(star_btn, star_icon_img);
    (b.gtk_button_set_has_frame)(star_btn, 0);
    (b.gtk_widget_set_size_request)(star_btn, 32, 32);
    let star_tip = CString::new("Bookmark this page").unwrap();
    (b.gtk_widget_set_tooltip_text)(star_btn, star_tip.as_ptr());
    (b.gtk_widget_add_css_class)(star_btn, class_action_btn.as_ptr());

    let cb_reader = std::mem::transmute::<unsafe extern "C" fn(*mut c_void, *mut c_void), GCallback>(
        on_reader_mode_clicked,
    );
    let cb_star = std::mem::transmute::<unsafe extern "C" fn(*mut c_void, *mut c_void), GCallback>(
        on_star_clicked,
    );

    (b.g_signal_connect_data)(
        reader_btn,
        CString::new("clicked").unwrap().as_ptr(),
        cb_reader,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
    );
    (b.g_signal_connect_data)(
        star_btn,
        CString::new("clicked").unwrap().as_ptr(),
        cb_star,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
    );

    (b.gtk_box_append)(title_box, lens_btn);
    (b.gtk_box_append)(title_box, entry);
    (b.gtk_box_append)(title_box, reader_btn);
    (b.gtk_box_append)(title_box, star_btn);
    (b.adw_header_bar_set_title_widget)(header, title_box);

    // Initialize TabManager
    if let Ok(mut mgr_guard) = TAB_MANAGER.lock() {
        let mut mgr = crate::tabs::TabManager::new();
        if let Some(tab) = mgr.active_tab_mut() {
            tab.url = start_url.clone();
            tab.is_start_page = is_initial_start_page;
            tab.title = if is_incognito {
                "Incognito Mode".to_string()
            } else if is_initial_start_page {
                "Start Page".to_string()
            } else {
                start_url.clone()
            };
            tab.is_incognito = is_incognito;
        }
        *mgr_guard = Some(mgr);
    }

    // Store active window pointer
    ACTIVE_WINDOW.store(win, Ordering::SeqCst);

    // Home button in HeaderBar (on left side)
    let home_btn =
        (b.gtk_button_new_from_icon_name)(CString::new("go-home-symbolic").unwrap().as_ptr());
    (b.gtk_button_set_has_frame)(home_btn, 0);
    (b.gtk_widget_set_size_request)(home_btn, 38, 38);
    let home_tip = CString::new("Open Start Page").unwrap();
    (b.gtk_widget_set_tooltip_text)(home_btn, home_tip.as_ptr());
    let cb_home = std::mem::transmute::<unsafe extern "C" fn(*mut c_void, *mut c_void), GCallback>(
        on_home_clicked,
    );
    (b.g_signal_connect_data)(
        home_btn,
        CString::new("clicked").unwrap().as_ptr(),
        cb_home,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
    );
    (b.adw_header_bar_pack_start)(header, home_btn);

    // Main Menu Button on top right of HeaderBar: opens side menu on the right
    let menu_btn =
        (b.gtk_button_new_from_icon_name)(CString::new("open-menu-symbolic").unwrap().as_ptr());
    (b.gtk_button_set_has_frame)(menu_btn, 0);
    (b.gtk_widget_set_size_request)(menu_btn, 38, 38);
    let menu_tip = CString::new("Browser Menu & Settings").unwrap();
    (b.gtk_widget_set_tooltip_text)(menu_btn, menu_tip.as_ptr());
    (b.adw_header_bar_pack_end)(header, menu_btn);

    // WebKit WebView
    let webview = (b.webkit_web_view_new)();
    ACTIVE_WEBVIEW.store(webview, Ordering::SeqCst);
    (b.gtk_widget_set_vexpand)(webview, 1);
    (b.gtk_widget_set_hexpand)(webview, 1);

    if !is_initial_start_page {
        if let Ok(c_start_url) = CString::new(start_url) {
            (b.webkit_web_view_load_uri)(webview, c_start_url.as_ptr());
        }
    }

    // Build Native Start Page widget
    let start_page = build_start_page_widget(b);
    apply_start_page_accent(&load_saved_accent());

    // Content Stack containing both start_page and webview
    let content_stack = (b.gtk_stack_new)();
    ACTIVE_CONTENT_STACK.store(content_stack, Ordering::SeqCst);
    (b.gtk_widget_set_vexpand)(content_stack, 1);
    (b.gtk_widget_set_hexpand)(content_stack, 1);

    (b.gtk_stack_add_named)(
        content_stack,
        start_page,
        CString::new("start_page").unwrap().as_ptr(),
    );
    (b.gtk_stack_add_named)(
        content_stack,
        webview,
        CString::new("webview").unwrap().as_ptr(),
    );

    if is_initial_start_page {
        (b.gtk_stack_set_visible_child_name)(
            content_stack,
            CString::new("start_page").unwrap().as_ptr(),
        );
        refresh_start_page();
    } else {
        (b.gtk_stack_set_visible_child_name)(
            content_stack,
            CString::new("webview").unwrap().as_ptr(),
        );
    }

    // Create Left Tab Sidebar container
    let sidebar_box = (b.gtk_box_new)(1, 0); // 1 = GTK_ORIENTATION_VERTICAL
    ACTIVE_SIDEBAR_BOX.store(sidebar_box, Ordering::SeqCst);
    (b.gtk_widget_set_size_request)(sidebar_box, MIN_COLLAPSED_SIDEBAR_WIDTH, -1);
    let sidebar_class = CString::new("sidebar").unwrap();
    (b.gtk_widget_add_css_class)(sidebar_box, sidebar_class.as_ptr());

    // Sidebar scrolling container
    let scroller = (b.gtk_scrolled_window_new)();
    (b.gtk_widget_set_vexpand)(scroller, 1);
    (b.gtk_scrolled_window_set_policy)(scroller, 2, 1); // H=Never (2), V=Automatic (1)

    let content_box = (b.gtk_box_new)(1, 4); // Vertical
    (b.gtk_widget_set_margin_start)(content_box, 4);
    (b.gtk_widget_set_margin_end)(content_box, 4);
    (b.gtk_widget_set_margin_top)(content_box, 6);
    (b.gtk_widget_set_margin_bottom)(content_box, 6);

    let pinned_box = (b.gtk_box_new)(1, 2); // Vertical
    ACTIVE_PINNED_BOX.store(pinned_box, Ordering::SeqCst);
    (b.gtk_box_append)(content_box, pinned_box);

    let sep = (b.gtk_separator_new)(0); // Horizontal separator
    ACTIVE_SEPARATOR.store(sep, Ordering::SeqCst);
    (b.gtk_widget_set_margin_top)(sep, 4);
    (b.gtk_widget_set_margin_bottom)(sep, 4);
    (b.gtk_box_append)(content_box, sep);

    let normal_box = (b.gtk_box_new)(1, 2); // Vertical
    ACTIVE_NORMAL_BOX.store(normal_box, Ordering::SeqCst);
    (b.gtk_box_append)(content_box, normal_box);

    (b.gtk_scrolled_window_set_child)(scroller, content_box);
    (b.gtk_box_append)(sidebar_box, scroller);

    // Resizable Paned container: sidebar on start, content_stack (start page / webview) on end
    let paned = (b.gtk_paned_new)(0); // 0 = GTK_ORIENTATION_HORIZONTAL
    ACTIVE_PANED.store(paned, Ordering::SeqCst);
    (b.gtk_widget_set_vexpand)(paned, 1);
    (b.gtk_widget_set_hexpand)(paned, 1);
    (b.gtk_paned_set_wide_handle)(paned, 1);
    (b.gtk_paned_set_start_child)(paned, sidebar_box);
    (b.gtk_paned_set_end_child)(paned, content_stack);
    (b.gtk_paned_set_resize_start_child)(paned, 0);
    (b.gtk_paned_set_shrink_start_child)(paned, 0);
    (b.gtk_paned_set_resize_end_child)(paned, 1);
    (b.gtk_paned_set_shrink_end_child)(paned, 1);
    (b.gtk_paned_set_position)(paned, 220);

    // Populate initial tab in sidebar
    refresh_sidebar_tabs();

    // -------------------------------------------------------------
    // Right Side Menu Construction (Grouped by functions)
    // -------------------------------------------------------------
    let right_sidebar = (b.gtk_box_new)(1, 0); // Vertical box
    ACTIVE_RIGHT_SIDEBAR.store(right_sidebar, Ordering::SeqCst);
    let class_right = CString::new("right-sidebar").unwrap();
    (b.gtk_widget_add_css_class)(right_sidebar, class_right.as_ptr());
    (b.gtk_widget_set_size_request)(right_sidebar, 380, -1);
    (b.gtk_widget_set_vexpand)(right_sidebar, 1);
    (b.gtk_widget_set_visible)(right_sidebar, 0); // Hidden initially until menu clicked

    // Header of the right side menu
    let right_header = (b.gtk_box_new)(0, 6); // Horizontal
    (b.gtk_widget_set_margin_start)(right_header, 12);
    (b.gtk_widget_set_margin_end)(right_header, 12);
    (b.gtk_widget_set_margin_top)(right_header, 10);
    (b.gtk_widget_set_margin_bottom)(right_header, 10);

    let lbl_menu_title =
        (b.gtk_label_new)(CString::new("Browser Menu & Settings").unwrap().as_ptr());
    (b.gtk_label_set_xalign)(lbl_menu_title, 0.0);
    (b.gtk_widget_set_hexpand)(lbl_menu_title, 1);
    let class_title = CString::new("menu-group-title").unwrap();
    (b.gtk_widget_add_css_class)(lbl_menu_title, class_title.as_ptr());

    let btn_close_menu = (b.gtk_button_new_with_label)(CString::new("✕").unwrap().as_ptr());
    (b.gtk_button_set_has_frame)(btn_close_menu, 0);
    let close_tip = CString::new("Close Menu").unwrap();
    (b.gtk_widget_set_tooltip_text)(btn_close_menu, close_tip.as_ptr());

    (b.gtk_box_append)(right_header, lbl_menu_title);
    (b.gtk_box_append)(right_header, btn_close_menu);
    (b.gtk_box_append)(right_sidebar, right_header);

    // Scrollable container for menu contents
    let scroller_right = (b.gtk_scrolled_window_new)();
    (b.gtk_widget_set_vexpand)(scroller_right, 1);
    (b.gtk_scrolled_window_set_policy)(scroller_right, 2, 1); // H=Never (2), V=Automatic (1)

    let panel_vbox = (b.gtk_box_new)(1, 8); // Vertical layout for options
    (b.gtk_widget_set_margin_bottom)(panel_vbox, 16);

    // Quick Action Buttons Row
    let quick_card = make_group_card(b, "⚡ Quick Actions");
    let quick_box = (b.gtk_box_new)(1, 4);
    let btn_side_new_tab =
        (b.gtk_button_new_with_label)(CString::new("➕  New Tab").unwrap().as_ptr());
    (b.gtk_button_set_has_frame)(btn_side_new_tab, 0);
    let btn_side_new_incognito =
        (b.gtk_button_new_with_label)(CString::new("🌊  New Incognito Window").unwrap().as_ptr());
    (b.gtk_button_set_has_frame)(btn_side_new_incognito, 0);
    let btn_side_history =
        (b.gtk_button_new_with_label)(CString::new("🕒  History & Bookmarks").unwrap().as_ptr());
    (b.gtk_button_set_has_frame)(btn_side_history, 0);
    let btn_side_about =
        (b.gtk_button_new_with_label)(CString::new("ℹ  About Vingilot").unwrap().as_ptr());
    (b.gtk_button_set_has_frame)(btn_side_about, 0);

    (b.gtk_box_append)(quick_box, btn_side_new_tab);
    (b.gtk_box_append)(quick_box, btn_side_new_incognito);
    (b.gtk_box_append)(quick_box, btn_side_history);
    (b.gtk_box_append)(quick_box, btn_side_about);
    (b.gtk_box_append)(quick_card, quick_box);
    (b.gtk_box_append)(panel_vbox, quick_card);

    // Group 1: Search Engines Preferences
    let card_search = make_group_card(b, "🔍 Search Engines Preferences");
    let lbl_search_desc = (b.gtk_label_new)(
        CString::new("Quick switch default search engine:")
            .unwrap()
            .as_ptr(),
    );
    (b.gtk_label_set_xalign)(lbl_search_desc, 0.0);
    (b.gtk_box_append)(card_search, lbl_search_desc);

    let search_btns_box = (b.gtk_box_new)(0, 4);
    let btn_ddg = (b.gtk_button_new_with_label)(CString::new("DuckDuckGo").unwrap().as_ptr());
    let btn_google = (b.gtk_button_new_with_label)(CString::new("Google").unwrap().as_ptr());
    let btn_brave = (b.gtk_button_new_with_label)(CString::new("Brave").unwrap().as_ptr());
    let btn_bing = (b.gtk_button_new_with_label)(CString::new("Bing").unwrap().as_ptr());
    let btn_ecosia = (b.gtk_button_new_with_label)(CString::new("Ecosia").unwrap().as_ptr());

    (b.gtk_box_append)(search_btns_box, btn_ddg);
    (b.gtk_box_append)(search_btns_box, btn_google);
    (b.gtk_box_append)(search_btns_box, btn_brave);
    (b.gtk_box_append)(search_btns_box, btn_bing);
    (b.gtk_box_append)(search_btns_box, btn_ecosia);
    (b.gtk_box_append)(card_search, search_btns_box);

    let (row_priv_search, _) = make_switch_row(b, "Encrypted & Private Queries", true);
    (b.gtk_box_append)(card_search, row_priv_search);
    (b.gtk_box_append)(panel_vbox, card_search);

    // Group 2: Theming
    let card_theme = make_group_card(b, "🎨 Theming");
    let lbl_theme_desc = (b.gtk_label_new)(
        CString::new("Color scheme and display themes:")
            .unwrap()
            .as_ptr(),
    );
    (b.gtk_label_set_xalign)(lbl_theme_desc, 0.0);
    (b.gtk_box_append)(card_theme, lbl_theme_desc);

    let theme_box1 = (b.gtk_box_new)(0, 4);
    let btn_theme_sys = (b.gtk_button_new_with_label)(CString::new("System").unwrap().as_ptr());
    let btn_theme_light = (b.gtk_button_new_with_label)(CString::new("Light").unwrap().as_ptr());
    let btn_theme_dark = (b.gtk_button_new_with_label)(CString::new("Dark").unwrap().as_ptr());
    let btn_theme_black =
        (b.gtk_button_new_with_label)(CString::new("True Black").unwrap().as_ptr());
    (b.gtk_box_append)(theme_box1, btn_theme_sys);
    (b.gtk_box_append)(theme_box1, btn_theme_light);
    (b.gtk_box_append)(theme_box1, btn_theme_dark);
    (b.gtk_box_append)(theme_box1, btn_theme_black);
    (b.gtk_box_append)(card_theme, theme_box1);

    let btn_theme_incog = (b.gtk_button_new_with_label)(
        CString::new("🌊 Ocean Deep Incognito (#062640)")
            .unwrap()
            .as_ptr(),
    );
    (b.gtk_box_append)(card_theme, btn_theme_incog);

    let lbl_accent_sel = (b.gtk_label_new)(CString::new("Start Page Accent:").unwrap().as_ptr());
    (b.gtk_label_set_xalign)(lbl_accent_sel, 0.0);
    (b.gtk_box_append)(card_theme, lbl_accent_sel);

    let accents_box1 = (b.gtk_box_new)(0, 4);
    let accents_box2 = (b.gtk_box_new)(0, 4);
    let cb_accent_btn = std::mem::transmute::<
        unsafe extern "C" fn(*mut c_void, *mut c_void),
        GCallback,
    >(on_accent_button_clicked);

    for (idx, (acc_name, _, _)) in GNOME_ACCENTS.iter().enumerate() {
        let acc_btn = (b.gtk_button_new_with_label)(CString::new(*acc_name).unwrap().as_ptr());
        (b.g_signal_connect_data)(
            acc_btn,
            CString::new("clicked").unwrap().as_ptr(),
            cb_accent_btn,
            idx as *mut c_void,
            std::ptr::null_mut(),
            0,
        );
        if idx < 5 {
            (b.gtk_box_append)(accents_box1, acc_btn);
        } else {
            (b.gtk_box_append)(accents_box2, acc_btn);
        }
    }
    (b.gtk_box_append)(card_theme, accents_box1);
    (b.gtk_box_append)(card_theme, accents_box2);

    let (row_reader_mode, _) = make_switch_row(b, "Auto Distraction-Free Reader Mode", true);
    (b.gtk_box_append)(card_theme, row_reader_mode);
    (b.gtk_box_append)(panel_vbox, card_theme);

    // Group 3: Webkit Personalisation
    let card_webkit = make_group_card(b, "🌐 Webkit Personalisation");
    let (row_js, _) = make_switch_row(b, "Enable JavaScript Execution", true);
    let (row_kinetic, _) = make_switch_row(b, "Kinetic Momentum Scrolling", true);
    let (row_cookies, _) = make_switch_row(b, "Strict Ephemeral Storage (Cookie Isolation)", true);
    let (row_trackers, _) = make_switch_row(b, "Enhanced Tracking Protection", true);
    let (row_article, _) = make_switch_row(b, "Auto Reader View Detection", true);
    (b.gtk_box_append)(card_webkit, row_js);
    (b.gtk_box_append)(card_webkit, row_kinetic);
    (b.gtk_box_append)(card_webkit, row_cookies);
    (b.gtk_box_append)(card_webkit, row_trackers);
    (b.gtk_box_append)(card_webkit, row_article);
    (b.gtk_box_append)(panel_vbox, card_webkit);

    // Group 4: HW Decoding and Acceleration
    let card_hw = make_group_card(b, "⚡ HW Decoding and Acceleration");
    let (row_hw_gpu, _) = make_switch_row(b, "Hardware GPU Acceleration", true);
    let (row_vulkan, _) = make_switch_row(b, "Vulkan Graphics Pipeline (GSK)", true);
    let (row_wayland, _) = make_switch_row(b, "Native Wayland Compositing", true);
    let (row_vcodec, _) = make_switch_row(b, "Hardware Video Decoding (VA-API/NVDEC)", true);
    let (row_webgl, _) = make_switch_row(b, "WebGL 2.0 & Canvas Hardware Rendering", true);
    (b.gtk_box_append)(card_hw, row_hw_gpu);
    (b.gtk_box_append)(card_hw, row_vulkan);
    (b.gtk_box_append)(card_hw, row_wayland);
    (b.gtk_box_append)(card_hw, row_vcodec);
    (b.gtk_box_append)(card_hw, row_webgl);
    (b.gtk_box_append)(panel_vbox, card_hw);

    // Group 5: AI Features (with editable fields)
    let card_ai = make_group_card(b, "🤖 AI Features");

    let lbl_ai_ep = (b.gtk_label_new)(CString::new("AI Provider Endpoint:").unwrap().as_ptr());
    (b.gtk_label_set_xalign)(lbl_ai_ep, 0.0);
    let entry_ai_ep = (b.gtk_entry_new)();
    ACTIVE_AI_BASE_URL_ENTRY.store(entry_ai_ep, Ordering::SeqCst);
    (b.gtk_entry_set_placeholder_text)(
        entry_ai_ep,
        CString::new("e.g. http://localhost:11434/v1 or https://api.openai.com/v1")
            .unwrap()
            .as_ptr(),
    );
    (b.gtk_editable_set_text)(
        entry_ai_ep,
        CString::new("http://localhost:11434/v1").unwrap().as_ptr(),
    );

    let lbl_ai_mod = (b.gtk_label_new)(CString::new("AI Model Name:").unwrap().as_ptr());
    (b.gtk_label_set_xalign)(lbl_ai_mod, 0.0);
    let entry_ai_mod = (b.gtk_entry_new)();
    ACTIVE_AI_MODEL_ENTRY.store(entry_ai_mod, Ordering::SeqCst);
    (b.gtk_entry_set_placeholder_text)(
        entry_ai_mod,
        CString::new("e.g. llama3.2, mistral, gpt-4o-mini")
            .unwrap()
            .as_ptr(),
    );
    (b.gtk_editable_set_text)(entry_ai_mod, CString::new("llama3.2").unwrap().as_ptr());

    let lbl_ai_key = (b.gtk_label_new)(CString::new("AI API Key (Optional):").unwrap().as_ptr());
    (b.gtk_label_set_xalign)(lbl_ai_key, 0.0);
    let entry_ai_key = (b.gtk_entry_new)();
    ACTIVE_AI_API_KEY_ENTRY.store(entry_ai_key, Ordering::SeqCst);
    (b.gtk_entry_set_placeholder_text)(
        entry_ai_key,
        CString::new("Enter API Key (sk-...)").unwrap().as_ptr(),
    );

    let (row_ai_sum, _) = make_switch_row(b, "Contextual Web Summarization", true);
    let (row_ai_mcp, _) = make_switch_row(b, "Agentic Navigation & MCP Dispatcher", true);

    let btn_save_ai = (b.gtk_button_new_with_label)(
        CString::new("💾  Apply & Save AI Configuration")
            .unwrap()
            .as_ptr(),
    );
    (b.gtk_widget_set_margin_top)(btn_save_ai, 6);

    let lbl_ai_status = (b.gtk_label_new)(CString::new("Ready to assist").unwrap().as_ptr());
    ACTIVE_AI_STATUS_LABEL.store(lbl_ai_status, Ordering::SeqCst);
    (b.gtk_label_set_xalign)(lbl_ai_status, 0.0);

    (b.gtk_box_append)(card_ai, lbl_ai_ep);
    (b.gtk_box_append)(card_ai, entry_ai_ep);
    (b.gtk_box_append)(card_ai, lbl_ai_mod);
    (b.gtk_box_append)(card_ai, entry_ai_mod);
    (b.gtk_box_append)(card_ai, lbl_ai_key);
    (b.gtk_box_append)(card_ai, entry_ai_key);
    (b.gtk_box_append)(card_ai, row_ai_sum);
    (b.gtk_box_append)(card_ai, row_ai_mcp);
    (b.gtk_box_append)(card_ai, btn_save_ai);
    (b.gtk_box_append)(card_ai, lbl_ai_status);
    (b.gtk_box_append)(panel_vbox, card_ai);

    (b.gtk_scrolled_window_set_child)(scroller_right, panel_vbox);
    (b.gtk_box_append)(right_sidebar, scroller_right);

    // Resizable Paned Container: Left Paned (Sidebar + WebView) & Right Side Menu
    let right_paned = (b.gtk_paned_new)(0); // 0 = GTK_ORIENTATION_HORIZONTAL
    ACTIVE_RIGHT_PANED.store(right_paned, Ordering::SeqCst);
    (b.gtk_widget_set_vexpand)(right_paned, 1);
    (b.gtk_widget_set_hexpand)(right_paned, 1);
    (b.gtk_paned_set_wide_handle)(right_paned, 1);
    (b.gtk_paned_set_start_child)(right_paned, paned);
    (b.gtk_paned_set_end_child)(right_paned, right_sidebar);
    (b.gtk_paned_set_resize_start_child)(right_paned, 1);
    (b.gtk_paned_set_shrink_start_child)(right_paned, 1);
    (b.gtk_paned_set_resize_end_child)(right_paned, 0);
    (b.gtk_paned_set_shrink_end_child)(right_paned, 1);

    // Connect signals
    let clicked_sig = CString::new("clicked").unwrap();
    let activate_sig = CString::new("activate").unwrap();
    let closed_sig = CString::new("closed").unwrap();
    let notify_active_sig = CString::new("notify::active").unwrap();
    let notify_pos_sig = CString::new("notify::position").unwrap();
    let notify_fav_sig = CString::new("notify::favicon").unwrap();
    let notify_title_sig = CString::new("notify::title").unwrap();
    let notify_uri_sig = CString::new("notify::uri").unwrap();

    let cb_sidebar = std::mem::transmute::<unsafe extern "C" fn(*mut c_void, *mut c_void), GCallback>(
        on_toggle_sidebar_clicked,
    );
    let cb_paned_notify = std::mem::transmute::<
        unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void),
        GCallback,
    >(on_paned_position_notify);
    let cb_fav_notify = std::mem::transmute::<
        unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void),
        GCallback,
    >(on_webview_favicon_notify);
    let cb_title_notify = std::mem::transmute::<
        unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void),
        GCallback,
    >(on_webview_title_notify);
    let cb_uri_notify = std::mem::transmute::<
        unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void),
        GCallback,
    >(on_webview_uri_notify);
    let cb_lens_closed = std::mem::transmute::<
        unsafe extern "C" fn(*mut c_void, *mut c_void),
        GCallback,
    >(on_lens_popover_closed);
    let cb_lens_notify = std::mem::transmute::<
        unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void),
        GCallback,
    >(on_lens_button_notify_active);

    let cb_back = std::mem::transmute::<unsafe extern "C" fn(*mut c_void, *mut c_void), GCallback>(
        on_back_clicked,
    );
    let cb_fwd = std::mem::transmute::<unsafe extern "C" fn(*mut c_void, *mut c_void), GCallback>(
        on_forward_clicked,
    );
    let cb_refresh = std::mem::transmute::<unsafe extern "C" fn(*mut c_void, *mut c_void), GCallback>(
        on_refresh_clicked,
    );
    let cb_entry = std::mem::transmute::<unsafe extern "C" fn(*mut c_void, *mut c_void), GCallback>(
        on_entry_activate,
    );

    let cb_new_tab = std::mem::transmute::<unsafe extern "C" fn(*mut c_void, *mut c_void), GCallback>(
        on_new_tab_clicked,
    );
    let cb_new_incognito = std::mem::transmute::<
        unsafe extern "C" fn(*mut c_void, *mut c_void),
        GCallback,
    >(on_new_incognito_clicked);
    let cb_zoom_out = std::mem::transmute::<
        unsafe extern "C" fn(*mut c_void, *mut c_void),
        GCallback,
    >(on_zoom_out_clicked);
    let cb_zoom_in = std::mem::transmute::<unsafe extern "C" fn(*mut c_void, *mut c_void), GCallback>(
        on_zoom_in_clicked,
    );
    let cb_zoom_reset = std::mem::transmute::<
        unsafe extern "C" fn(*mut c_void, *mut c_void),
        GCallback,
    >(on_zoom_reset_clicked);
    let cb_history = std::mem::transmute::<unsafe extern "C" fn(*mut c_void, *mut c_void), GCallback>(
        on_history_clicked,
    );
    let cb_about = std::mem::transmute::<unsafe extern "C" fn(*mut c_void, *mut c_void), GCallback>(
        on_about_clicked,
    );

    let cb_toggle_menu = std::mem::transmute::<
        unsafe extern "C" fn(*mut c_void, *mut c_void),
        GCallback,
    >(on_toggle_right_menu_clicked);
    let cb_close_menu = std::mem::transmute::<
        unsafe extern "C" fn(*mut c_void, *mut c_void),
        GCallback,
    >(on_close_right_menu_clicked);
    let cb_save_ai = std::mem::transmute::<unsafe extern "C" fn(*mut c_void, *mut c_void), GCallback>(
        on_save_ai_settings_clicked,
    );

    let cb_search_ddg = std::mem::transmute::<
        unsafe extern "C" fn(*mut c_void, *mut c_void),
        GCallback,
    >(on_search_ddg_clicked);
    let cb_search_google = std::mem::transmute::<
        unsafe extern "C" fn(*mut c_void, *mut c_void),
        GCallback,
    >(on_search_google_clicked);
    let cb_search_brave = std::mem::transmute::<
        unsafe extern "C" fn(*mut c_void, *mut c_void),
        GCallback,
    >(on_search_brave_clicked);
    let cb_search_bing = std::mem::transmute::<
        unsafe extern "C" fn(*mut c_void, *mut c_void),
        GCallback,
    >(on_search_bing_clicked);
    let cb_search_ecosia = std::mem::transmute::<
        unsafe extern "C" fn(*mut c_void, *mut c_void),
        GCallback,
    >(on_search_ecosia_clicked);

    let cb_theme_sys = std::mem::transmute::<
        unsafe extern "C" fn(*mut c_void, *mut c_void),
        GCallback,
    >(on_theme_system_clicked);
    let cb_theme_light = std::mem::transmute::<
        unsafe extern "C" fn(*mut c_void, *mut c_void),
        GCallback,
    >(on_theme_light_clicked);
    let cb_theme_dark = std::mem::transmute::<
        unsafe extern "C" fn(*mut c_void, *mut c_void),
        GCallback,
    >(on_theme_dark_clicked);
    let cb_theme_black = std::mem::transmute::<
        unsafe extern "C" fn(*mut c_void, *mut c_void),
        GCallback,
    >(on_theme_true_black_clicked);
    let cb_theme_incog = std::mem::transmute::<
        unsafe extern "C" fn(*mut c_void, *mut c_void),
        GCallback,
    >(on_theme_incognito_blue_clicked);

    // Menu toggle and close signals
    (b.g_signal_connect_data)(
        menu_btn,
        clicked_sig.as_ptr(),
        cb_toggle_menu,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
    );
    (b.g_signal_connect_data)(
        btn_close_menu,
        clicked_sig.as_ptr(),
        cb_close_menu,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
    );

    // Quick action signals in side menu
    (b.g_signal_connect_data)(
        btn_side_new_tab,
        clicked_sig.as_ptr(),
        cb_new_tab,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
    );
    (b.g_signal_connect_data)(
        btn_side_new_incognito,
        clicked_sig.as_ptr(),
        cb_new_incognito,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
    );
    (b.g_signal_connect_data)(
        btn_side_history,
        clicked_sig.as_ptr(),
        cb_history,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
    );
    (b.g_signal_connect_data)(
        btn_side_about,
        clicked_sig.as_ptr(),
        cb_about,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
    );

    // Search Engine signals in side menu
    (b.g_signal_connect_data)(
        btn_ddg,
        clicked_sig.as_ptr(),
        cb_search_ddg,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
    );
    (b.g_signal_connect_data)(
        btn_google,
        clicked_sig.as_ptr(),
        cb_search_google,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
    );
    (b.g_signal_connect_data)(
        btn_brave,
        clicked_sig.as_ptr(),
        cb_search_brave,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
    );
    (b.g_signal_connect_data)(
        btn_bing,
        clicked_sig.as_ptr(),
        cb_search_bing,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
    );
    (b.g_signal_connect_data)(
        btn_ecosia,
        clicked_sig.as_ptr(),
        cb_search_ecosia,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
    );

    // Theme signals in side menu
    (b.g_signal_connect_data)(
        btn_theme_sys,
        clicked_sig.as_ptr(),
        cb_theme_sys,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
    );
    (b.g_signal_connect_data)(
        btn_theme_light,
        clicked_sig.as_ptr(),
        cb_theme_light,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
    );
    (b.g_signal_connect_data)(
        btn_theme_dark,
        clicked_sig.as_ptr(),
        cb_theme_dark,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
    );
    (b.g_signal_connect_data)(
        btn_theme_black,
        clicked_sig.as_ptr(),
        cb_theme_black,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
    );
    (b.g_signal_connect_data)(
        btn_theme_incog,
        clicked_sig.as_ptr(),
        cb_theme_incog,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
    );

    // AI save signal in side menu
    (b.g_signal_connect_data)(
        btn_save_ai,
        clicked_sig.as_ptr(),
        cb_save_ai,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
    );

    // Sidebar signals (when present in normal mode)
    if !btn_sidebar.is_null() {
        (b.g_signal_connect_data)(
            btn_sidebar,
            clicked_sig.as_ptr(),
            cb_sidebar,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            0,
        );
    }
    (b.g_signal_connect_data)(
        paned,
        notify_pos_sig.as_ptr(),
        cb_paned_notify,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
    );
    (b.g_signal_connect_data)(
        webview,
        notify_fav_sig.as_ptr(),
        cb_fav_notify,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
    );
    (b.g_signal_connect_data)(
        webview,
        notify_title_sig.as_ptr(),
        cb_title_notify,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
    );
    (b.g_signal_connect_data)(
        webview,
        notify_uri_sig.as_ptr(),
        cb_uri_notify,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
    );

    // Lens zoom signals
    (b.g_signal_connect_data)(
        lens_btn,
        notify_active_sig.as_ptr(),
        cb_lens_notify,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
    );
    (b.g_signal_connect_data)(
        lens_popover,
        closed_sig.as_ptr(),
        cb_lens_closed,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
    );
    (b.g_signal_connect_data)(
        btn_lens_out,
        clicked_sig.as_ptr(),
        cb_zoom_out,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
    );
    (b.g_signal_connect_data)(
        btn_lens_in,
        clicked_sig.as_ptr(),
        cb_zoom_in,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
    );
    (b.g_signal_connect_data)(
        btn_lens_reset,
        clicked_sig.as_ptr(),
        cb_zoom_reset,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
    );

    // Nav signals (when present in normal mode)
    if !btn_back.is_null() {
        (b.g_signal_connect_data)(
            btn_back,
            clicked_sig.as_ptr(),
            cb_back,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            0,
        );
    }
    if !btn_fwd.is_null() {
        (b.g_signal_connect_data)(
            btn_fwd,
            clicked_sig.as_ptr(),
            cb_fwd,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            0,
        );
    }
    if !btn_refresh.is_null() {
        (b.g_signal_connect_data)(
            btn_refresh,
            clicked_sig.as_ptr(),
            cb_refresh,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            0,
        );
    }
    (b.g_signal_connect_data)(
        entry,
        activate_sig.as_ptr(),
        cb_entry,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
    );

    // Box container: HeaderBar on top, right_paned (paned + right side menu) below
    let box_vertical = (b.gtk_box_new)(1, 0); // 1 = GTK_ORIENTATION_VERTICAL
    (b.gtk_box_append)(box_vertical, header);
    (b.gtk_box_append)(box_vertical, right_paned);

    (b.adw_application_window_set_content)(win, box_vertical);
    (b.gtk_window_present)(win);
}

pub fn run_gui(is_incognito: bool, initial_url: Option<String>) -> bool {
    let has_display = std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok();
    if !has_display {
        println!("No graphical display found (DISPLAY and WAYLAND_DISPLAY are unset).");
        return false;
    }

    unsafe {
        let bindings = match load_bindings() {
            Some(b) => b,
            None => {
                println!(
                    "GTK4 / Libadwaita / WebKitGTK 6.0 dynamic libraries not found on system."
                );
                return false;
            }
        };

        IS_INCOGNITO.store(is_incognito, Ordering::SeqCst);
        if let Ok(mut url_guard) = INITIAL_URL.lock() {
            *url_guard = initial_url;
        }
        let _ = BINDINGS.set(bindings);
        let b = BINDINGS.get().unwrap();
        (b.adw_init)();

        // Register custom icon search path (for symbolic icons like hamburger)
        let display = (b.gdk_display_get_default)();
        if !display.is_null() {
            let icon_theme = (b.gtk_icon_theme_get_for_display)(display);
            if !icon_theme.is_null() {
                let local_icons_path = CString::new("data/icons").unwrap();
                (b.gtk_icon_theme_add_search_path)(icon_theme, local_icons_path.as_ptr());
                let app_icons_path = CString::new("/app/share/icons").unwrap();
                (b.gtk_icon_theme_add_search_path)(icon_theme, app_icons_path.as_ptr());
            }
        }

        let app_id = CString::new("io.github.antux912012.Vingilot").unwrap();
        let app = (b.adw_application_new)(app_id.as_ptr(), 0);
        if app.is_null() {
            return false;
        }

        let activate_sig = CString::new("activate").unwrap();
        let cb_activate = std::mem::transmute::<
            unsafe extern "C" fn(*mut c_void, *mut c_void),
            GCallback,
        >(on_app_activate);

        (b.g_signal_connect_data)(
            app,
            activate_sig.as_ptr(),
            cb_activate,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            0,
        );

        println!("[Vingilot] Starting GApplication main event loop...");
        let status = (b.g_application_run)(app, 0, std::ptr::null());
        (b.g_object_unref)(app);

        status == 0
    }
}
