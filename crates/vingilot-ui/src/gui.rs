use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use std::sync::{Mutex, OnceLock};

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
static SIDEBAR_COLLAPSED: AtomicBool = AtomicBool::new(false);
static TAB_MANAGER: Mutex<Option<crate::tabs::TabManager>> = Mutex::new(None);

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

        g_signal_connect_data: load_symbol(libgobj, "g_signal_connect_data")?,
        g_application_run: load_symbol(libgio, "g_application_run")?,
        g_object_unref: load_symbol(libgobj, "g_object_unref")?,
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
    let webview = ACTIVE_WEBVIEW.load(Ordering::SeqCst);
    if webview.is_null() || entry.is_null() {
        return;
    }
    if let Some(b) = BINDINGS.get() {
        let text_ptr = (b.gtk_editable_get_text)(entry);
        if !text_ptr.is_null() {
            let text = CStr::from_ptr(text_ptr)
                .to_string_lossy()
                .trim()
                .to_string();
            let url = if text.starts_with("http://")
                || text.starts_with("https://")
                || text.starts_with("about:")
                || text.starts_with("file://")
            {
                text
            } else if text.contains('.') && !text.contains(' ') {
                format!("https://{}", text)
            } else {
                format!("https://duckduckgo.com/?q={}", text)
            };

            if let Ok(c_url) = CString::new(url) {
                (b.webkit_web_view_load_uri)(webview, c_url.as_ptr());
            }
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
            if let Ok(mut mgr_guard) = TAB_MANAGER.lock() {
                if let Some(mgr) = mgr_guard.as_mut() {
                    if let Some(active) = mgr.active_tab_mut() {
                        if !active.is_incognito {
                            active.title = title_str;
                        }
                    }
                }
            }
            refresh_sidebar_tabs();
        }
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
                    active.url = uri_str;
                }
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
            let img = if tab.favicon_ptr != 0 {
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
                            "Navigating Shadowy Seas".to_string()
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

            if mgr.switch_tab(idx) {
                if let Some(tab) = mgr.active_tab() {
                    url_to_load = Some(tab.url.clone());
                }
            }
        }
    }

    if let Some(url) = url_to_load {
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
    if let Ok(mut mgr_guard) = TAB_MANAGER.lock() {
        if let Some(mgr) = mgr_guard.as_mut() {
            mgr.close_tab(idx);
            if let Some(tab) = mgr.active_tab() {
                url_to_load = Some(tab.url.clone());
            }
        }
    }

    if let Some(url) = url_to_load {
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
    let b = match BINDINGS.get() {
        Some(b) => b,
        None => return,
    };
    let webview = ACTIVE_WEBVIEW.load(Ordering::SeqCst);
    let entry = ACTIVE_ENTRY.load(Ordering::SeqCst);

    let home = "https://duckduckgo.com".to_string();
    if let Ok(mut mgr_guard) = TAB_MANAGER.lock() {
        if let Some(mgr) = mgr_guard.as_mut() {
            // Save active tab state
            if !webview.is_null() {
                let current_uri_ptr = (b.webkit_web_view_get_uri)(webview);
                if !current_uri_ptr.is_null() {
                    let cur_uri = CStr::from_ptr(current_uri_ptr)
                        .to_string_lossy()
                        .to_string();
                    if let Some(active) = mgr.active_tab_mut() {
                        active.url = cur_uri.clone();
                    }
                }
            }

            mgr.new_tab();
            if let Some(active) = mgr.active_tab_mut() {
                active.url = home.clone();
                active.title = "DuckDuckGo".to_string();
            }
        }
    }

    if !webview.is_null() {
        let c_home = CString::new(home.clone()).unwrap();
        (b.webkit_web_view_load_uri)(webview, c_home.as_ptr());
    }
    if !entry.is_null() {
        let c_home = CString::new(home).unwrap();
        (b.gtk_editable_set_text)(entry, c_home.as_ptr());
    }

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
        (b.gtk_paned_set_position)(paned, 48);
        (b.gtk_widget_set_size_request)(sidebar, 48, -1);
    } else {
        (b.gtk_paned_set_position)(paned, 220);
        (b.gtk_widget_set_size_request)(sidebar, 48, -1);
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

    let row_shadowy = (b.adw_switch_row_new)();
    (b.adw_preferences_row_set_title)(
        row_shadowy,
        CString::new("Shadowy Seas Incognito Isolation")
            .unwrap()
            .as_ptr(),
    );
    (b.adw_action_row_set_subtitle)(
        row_shadowy,
        CString::new("Zero cache and ephemeral memory storage in private sessions")
            .unwrap()
            .as_ptr(),
    );
    (b.adw_switch_row_set_active)(row_shadowy, 1);
    (b.adw_preferences_group_add)(grp_priv, row_shadowy);

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
    (b.adw_about_dialog_set_version)(dialog, CString::new("0.1.1a").unwrap().as_ptr());
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

unsafe extern "C" fn on_app_activate(app: *mut c_void, _user_data: *mut c_void) {
    println!("[Vingilot] GApplication activate signal received - initializing Adwaita window");
    let b = match BINDINGS.get() {
        Some(bindings) => bindings,
        None => return,
    };

    let win = (b.adw_application_window_new)(app);
    let is_incognito = IS_INCOGNITO.load(Ordering::SeqCst);

    let title = if is_incognito {
        CString::new("Vingilot — Navigating Shadowy Seas (Incognito)").unwrap()
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
    }

    // Header bar
    let header = (b.adw_header_bar_new)();

    // Sidebar toggle button (3 horizontal lines hamburger menu)
    let hamburger_icon = CString::new("open-menu-symbolic").unwrap();
    let btn_sidebar = (b.gtk_button_new_from_icon_name)(hamburger_icon.as_ptr());
    let sidebar_tip = CString::new("Toggle Tab Sidebar").unwrap();
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

    let start_url = INITIAL_URL
        .lock()
        .ok()
        .and_then(|g| g.clone())
        .unwrap_or_else(|| "https://duckduckgo.com".to_string());

    if let Ok(c_start_url) = CString::new(start_url.clone()) {
        (b.gtk_editable_set_text)(entry, c_start_url.as_ptr());
    }

    (b.gtk_box_append)(title_box, lens_btn);
    (b.gtk_box_append)(title_box, entry);
    (b.adw_header_bar_set_title_widget)(header, title_box);

    // Initialize TabManager
    if let Ok(mut mgr_guard) = TAB_MANAGER.lock() {
        let mut mgr = crate::tabs::TabManager::new();
        if let Some(tab) = mgr.active_tab_mut() {
            tab.url = start_url.clone();
            tab.title = if is_incognito {
                "Navigating Shadowy Seas".to_string()
            } else {
                "DuckDuckGo".to_string()
            };
            tab.is_incognito = is_incognito;
        }
        *mgr_guard = Some(mgr);
    }

    // Store active window pointer
    ACTIVE_WINDOW.store(win, Ordering::SeqCst);

    // 3-dots Menu Button on top right (on same line as window buttons)
    let menu_btn = (b.gtk_menu_button_new)();
    let menu_icon = CString::new("view-more-symbolic").unwrap();
    (b.gtk_menu_button_set_icon_name)(menu_btn, menu_icon.as_ptr());
    let menu_tip = CString::new("Main Menu").unwrap();
    (b.gtk_widget_set_tooltip_text)(menu_btn, menu_tip.as_ptr());

    let popover = (b.gtk_popover_new)();
    ACTIVE_POPOVER.store(popover, Ordering::SeqCst);
    (b.gtk_menu_button_set_popover)(menu_btn, popover);

    // Popover vertical layout container
    let menu_box = (b.gtk_box_new)(1, 4); // 1 = GTK_ORIENTATION_VERTICAL, 4px spacing
    (b.gtk_widget_set_margin_start)(menu_box, 8);
    (b.gtk_widget_set_margin_end)(menu_box, 8);
    (b.gtk_widget_set_margin_top)(menu_box, 8);
    (b.gtk_widget_set_margin_bottom)(menu_box, 8);
    (b.gtk_popover_set_child)(popover, menu_box);

    // 1. New Tab
    let btn_new_tab = (b.gtk_button_new_with_label)(CString::new("➕  New Tab").unwrap().as_ptr());
    (b.gtk_button_set_has_frame)(btn_new_tab, 0);
    (b.gtk_box_append)(menu_box, btn_new_tab);

    // 2. New Incognito Window
    let btn_new_incognito =
        (b.gtk_button_new_with_label)(CString::new("🌊  New Incognito Window").unwrap().as_ptr());
    (b.gtk_button_set_has_frame)(btn_new_incognito, 0);
    (b.gtk_box_append)(menu_box, btn_new_incognito);

    // Separator
    let sep1 = (b.gtk_separator_new)(0);
    (b.gtk_box_append)(menu_box, sep1);

    // 3. History & Bookmarks
    let btn_history =
        (b.gtk_button_new_with_label)(CString::new("🕒  History & Bookmarks").unwrap().as_ptr());
    (b.gtk_button_set_has_frame)(btn_history, 0);
    (b.gtk_box_append)(menu_box, btn_history);

    // Separator
    let sep2 = (b.gtk_separator_new)(0);
    (b.gtk_box_append)(menu_box, sep2);

    // 4. Preferences
    let btn_prefs = (b.gtk_button_new_with_label)(CString::new("⚙  Preferences").unwrap().as_ptr());
    (b.gtk_button_set_has_frame)(btn_prefs, 0);
    (b.gtk_box_append)(menu_box, btn_prefs);

    // 5. About Vingilot
    let btn_about =
        (b.gtk_button_new_with_label)(CString::new("ℹ  About Vingilot").unwrap().as_ptr());
    (b.gtk_button_set_has_frame)(btn_about, 0);
    (b.gtk_box_append)(menu_box, btn_about);

    // Pack 3-dots Menu Button to top-right
    (b.adw_header_bar_pack_end)(header, menu_btn);

    // Incognito indicator
    if is_incognito {
        let badge_label = CString::new("🌊 Shadowy Seas").unwrap();
        let btn_badge = (b.gtk_button_new_with_label)(badge_label.as_ptr());
        (b.adw_header_bar_pack_end)(header, btn_badge);
    }

    // WebKit WebView
    let webview = (b.webkit_web_view_new)();
    ACTIVE_WEBVIEW.store(webview, Ordering::SeqCst);
    (b.gtk_widget_set_vexpand)(webview, 1);
    (b.gtk_widget_set_hexpand)(webview, 1);

    if let Ok(c_start_url) = CString::new(start_url) {
        (b.webkit_web_view_load_uri)(webview, c_start_url.as_ptr());
    }

    // Create Sidebar container (left side)
    let sidebar_box = (b.gtk_box_new)(1, 0); // 1 = GTK_ORIENTATION_VERTICAL
    ACTIVE_SIDEBAR_BOX.store(sidebar_box, Ordering::SeqCst);
    (b.gtk_widget_set_size_request)(sidebar_box, 48, -1);
    let sidebar_class = CString::new("sidebar").unwrap();
    (b.gtk_widget_add_css_class)(sidebar_box, sidebar_class.as_ptr());

    // Sidebar scrolling container
    let scroller = (b.gtk_scrolled_window_new)();
    (b.gtk_widget_set_vexpand)(scroller, 1);
    (b.gtk_scrolled_window_set_policy)(scroller, 1, 2); // H=Never, V=Automatic

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

    // Resizable Paned container: sidebar on start, webview on end
    let paned = (b.gtk_paned_new)(0); // 0 = GTK_ORIENTATION_HORIZONTAL
    ACTIVE_PANED.store(paned, Ordering::SeqCst);
    (b.gtk_widget_set_vexpand)(paned, 1);
    (b.gtk_widget_set_hexpand)(paned, 1);
    (b.gtk_paned_set_wide_handle)(paned, 1);
    (b.gtk_paned_set_start_child)(paned, sidebar_box);
    (b.gtk_paned_set_end_child)(paned, webview);
    (b.gtk_paned_set_resize_start_child)(paned, 0);
    (b.gtk_paned_set_shrink_start_child)(paned, 0);
    (b.gtk_paned_set_resize_end_child)(paned, 1);
    (b.gtk_paned_set_shrink_end_child)(paned, 1);
    (b.gtk_paned_set_position)(paned, 220);

    // Populate initial tab in sidebar
    refresh_sidebar_tabs();

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
    let cb_prefs = std::mem::transmute::<unsafe extern "C" fn(*mut c_void, *mut c_void), GCallback>(
        on_preferences_clicked,
    );
    let cb_about = std::mem::transmute::<unsafe extern "C" fn(*mut c_void, *mut c_void), GCallback>(
        on_about_clicked,
    );

    // Sidebar signals
    (b.g_signal_connect_data)(
        btn_sidebar,
        clicked_sig.as_ptr(),
        cb_sidebar,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
    );
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

    // Nav signals
    (b.g_signal_connect_data)(
        btn_back,
        clicked_sig.as_ptr(),
        cb_back,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
    );
    (b.g_signal_connect_data)(
        btn_fwd,
        clicked_sig.as_ptr(),
        cb_fwd,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
    );
    (b.g_signal_connect_data)(
        btn_refresh,
        clicked_sig.as_ptr(),
        cb_refresh,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
    );
    (b.g_signal_connect_data)(
        entry,
        activate_sig.as_ptr(),
        cb_entry,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
    );

    // 3-dots Menu signals
    (b.g_signal_connect_data)(
        btn_new_tab,
        clicked_sig.as_ptr(),
        cb_new_tab,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
    );
    (b.g_signal_connect_data)(
        btn_new_incognito,
        clicked_sig.as_ptr(),
        cb_new_incognito,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
    );
    (b.g_signal_connect_data)(
        btn_history,
        clicked_sig.as_ptr(),
        cb_history,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
    );
    (b.g_signal_connect_data)(
        btn_prefs,
        clicked_sig.as_ptr(),
        cb_prefs,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
    );
    (b.g_signal_connect_data)(
        btn_about,
        clicked_sig.as_ptr(),
        cb_about,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
    );

    // Box container: HeaderBar on top, paned (sidebar + webview) below
    let box_vertical = (b.gtk_box_new)(1, 0); // 1 = GTK_ORIENTATION_VERTICAL
    (b.gtk_box_append)(box_vertical, header);
    (b.gtk_box_append)(box_vertical, paned);

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
