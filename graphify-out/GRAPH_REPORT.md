# Graph Report - venu---Dynamic-Notch-for-Windows  (2026-11-19)

## Corpus Check
- cluster-only mode — file stats not available

## Summary
- 2463 nodes · 5671 edges · 142 communities (114 shown, 28 thin omitted)
- Extraction: 98% EXTRACTED · 2% INFERRED · 0% AMBIGUOUS · INFERRED: 125 edges (avg confidence: 0.77)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `92e4442d`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- Community 0
- Community 1
- Community 2
- Community 3
- Community 4
- Community 5
- Community 6
- Community 7
- Community 8
- Community 9
- Community 10
- Community 11
- Community 12
- Community 13
- Community 14
- Community 15
- Community 16
- Community 17
- Community 18
- Community 19
- Community 20
- Community 21
- Community 22
- Community 23
- Community 24
- Community 25
- Community 26
- Community 27
- Community 28
- Community 29
- Community 30
- Community 31
- Community 32
- Community 33
- Community 34
- Community 35
- Community 36
- Community 37
- Community 38
- Community 39
- Community 40
- Community 41
- Community 42
- Community 43
- Community 44
- Community 45
- Community 46
- Community 47
- Community 48
- Community 49
- Community 50
- Community 51
- Community 52
- Community 53
- Community 54
- Community 55
- Community 56
- Community 57
- Community 58
- Community 59
- Community 60
- Community 61
- Community 62
- Community 63
- Community 64
- Community 65
- Community 66
- Community 67
- Community 68
- Community 69
- Community 70
- Community 71
- Community 72
- Community 73
- Community 74
- Community 75
- Community 76
- Community 77
- Community 78
- Community 79
- Community 80
- Community 81
- Community 82
- Community 83
- Community 84
- Community 85
- Community 86
- Community 87
- Community 88
- Community 89
- Community 90
- Community 91
- Community 92
- Community 93
- Community 94
- Community 95
- Community 96
- Community 97
- Community 98
- Community 99
- Community 100
- Community 101
- Community 102
- Community 103
- Community 104
- Community 105
- Community 106
- Community 107
- Community 108
- Community 109
- Community 110
- Community 111
- Community 112
- Community 113
- Community 114
- Community 115
- Community 116
- Community 117
- Community 118
- Community 119
- Community 121
- Community 122
- Community 123
- Community 124
- Community 125
- Community 126
- Community 127
- Community 130
- Community 131
- Community 132
- Community 133

## God Nodes (most connected - your core abstractions)
1. `AppConfig` - 169 edges
2. `SettingsApp` - 102 edges
3. `Painter` - 78 edges
4. `NotchState` - 70 edges
5. `SlideKind` - 58 edges
6. `NotchShape` - 56 edges
7. `NotchWindow` - 52 edges
8. `PageCtx` - 46 edges
9. `Rect` - 33 edges
10. `FlashConfig` - 31 edges

## Surprising Connections (you probably didn't know these)
- `DANGER` --references--> `Rgba`  [INFERRED]
  src/notch/paint_panels.rs → src/notch/theme.rs
- `draw_checkerboard()` --references--> `Painter`  [INFERRED]
  src/gui/color_picker.rs → src/notch/paint.rs
- `paint_glyph()` --references--> `Painter`  [INFERRED]
  src/gui/mod.rs → src/notch/paint.rs
- `paint_icon()` --references--> `Painter`  [INFERRED]
  src/gui/mod.rs → src/notch/paint.rs
- `paint_desktop()` --references--> `Painter`  [INFERRED]
  src/gui/notch_preview.rs → src/notch/paint.rs

## Import Cycles
- 1-file cycle: `src/native_notifications.rs -> src/native_notifications.rs`
- 1-file cycle: `src/tray.rs -> src/tray.rs`
- 1-file cycle: `src/notch/geom.rs -> src/notch/geom.rs`
- 1-file cycle: `src/notch/notify.rs -> src/notch/notify.rs`
- 1-file cycle: `src/launch.rs -> src/launch.rs`
- 1-file cycle: `src/gui/filedlg.rs -> src/gui/filedlg.rs`
- 2-file cycle: `src/notch/paint.rs -> src/notch/paint_panels.rs -> src/notch/paint.rs`

## Communities (142 total, 28 thin omitted)

### Community 0 - "Community 0"
Cohesion: 0.03
Nodes (52): ACCENT_PRESETS, ease_out(), EGUI_CTX, enum_windows_callback(), find_settings_hwnd(), Glyph, Close, Minimize (+44 more)

### Community 1 - "Community 1"
Cohesion: 0.05
Nodes (28): install_panic_hook(), local_data_directory(), LOG_LOCK, MAX_LOG_BYTES, record_eframe_error(), record_panic(), sanitize_diagnostic(), write_record() (+20 more)

### Community 2 - "Community 2"
Cohesion: 0.05
Nodes (33): DEMAND_LEASE, filetime_ticks(), GPU_COUNTER, GPU_SAMPLE_EVERY, GpuQuery, busiest_gpu_engine(), byte_format_is_compact(), cpu_uses_consecutive_global_times_and_keeps_warmup_unavailable() (+25 more)

### Community 3 - "Community 3"
Cohesion: 0.05
Nodes (27): bundled_families_resolve_case_insensitively_and_fall_back_safely(), BUNDLED_FAMILY_NAMES, family_for_collection(), GEIST, GEIST_MONO, is_bundled_family(), SYSTEM_FALLBACK, build_private_collection() (+19 more)

### Community 4 - "Community 4"
Cohesion: 0.06
Nodes (32): command_for(), command_is_quoted_and_quiet(), disable(), enable(), is_registered(), managed_by_windows(), on_winrt_thread(), query_value() (+24 more)

### Community 5 - "Community 5"
Cohesion: 0.04
Nodes (23): AMBIENT_FRAME, CLASS_NAME, CLASS_REGISTERED, FLYBY_SPEED, FULLSCREEN_REFRESH, HOTKEY_CLICK_THROUGH, HOTKEY_DOWN, HOTKEY_ENTER (+15 more)

### Community 6 - "Community 6"
Cohesion: 0.05
Nodes (14): WAVE_BARS, BREAK_GREEN, clock_key(), MONTHS, clock_text(), DANGER, GLYPH_PASTE, GLYPH_PIN (+6 more)

### Community 7 - "Community 7"
Cohesion: 0.05
Nodes (11): ID_TRAY_CLICK_THROUGH, ID_TRAY_EXIT, ID_TRAY_OPEN_NOTCH, ID_TRAY_RESET, ID_TRAY_SETTINGS, SETTINGS_REQUEST_SENDER, SHOW_REQUESTED, TRAY_CLASS_NAME (+3 more)

### Community 8 - "Community 8"
Cohesion: 0.08
Nodes (4): CLASS_NAME, CLASS_REGISTERED, CLASS_NAME, CLASS_REGISTERED

### Community 9 - "Community 9"
Cohesion: 0.10
Nodes (11): revision(), clear(), copy_text(), pill_media_buttons(), activity_revision(), PodDraw, Scene, FrameKey (+3 more)

### Community 10 - "Community 10"
Cohesion: 0.12
Nodes (7): build_3sided_border_geometry(), CachedArt, CachedImage, IconCache, outside(), Painter, scale_matrix()

### Community 11 - "Community 11"
Cohesion: 0.13
Nodes (4): pick_image(), PageCtx, SettingsApp, .SUGGESTED_APPS

### Community 12 - "Community 12"
Cohesion: 0.07
Nodes (38): PodsConfig, .MAX_PODS, a_face_on_the_pill_suppresses_its_pod(), CLIP_HEADER, CLIP_ROW_GAP, CLIP_ROW_H, disabled_sources_are_skipped(), everything() (+30 more)

### Community 13 - "Community 13"
Cohesion: 0.11
Nodes (4): BG_RADIUS, BLOCK_GAP, scale_matrix(), scale_xy()

### Community 14 - "Community 14"
Cohesion: 0.10
Nodes (35): civil_from_days(), civil_round_trip(), configure(), days_from_civil(), durations(), Event, expand(), fetch() (+27 more)

### Community 15 - "Community 15"
Cohesion: 0.09
Nodes (26): BorderEffectConfig, .CONTINUOUS_FPS, .MAX_INTERVAL_SECS, .MIN_INTERVAL_SECS, AURORA, Axis, bump(), Draw (+18 more)

### Community 17 - "Community 17"
Cohesion: 0.12
Nodes (30): text_segment_width(), a_click_keeps_the_panel_open_until_it_is_released(), a_click_ripple_runs_once_and_the_hover_fades_out(), border_effect_off_never_runs(), border_effect_runs_once_then_waits_its_interval(), border_effect_waits_for_a_notification_glow(), closing_on_purpose_keeps_hover_from_reopening_it(), continuous_effect_loops_without_a_break() (+22 more)

### Community 18 - "Community 18"
Cohesion: 0.05
Nodes (9): CF_DIB, CF_HDROP, CF_UNICODETEXT, latest(), MAX_CHARS, MAX_IMAGE_SIDE, one_line(), revision() (+1 more)

### Community 19 - "Community 19"
Cohesion: 0.06
Nodes (14): CONNECTED, DENIED, DISABLED, ERROR, is_connected(), NEEDS_CONSENT, NEEDS_IDENTITY, REQUESTED_ENABLED (+6 more)

### Community 20 - "Community 20"
Cohesion: 0.07
Nodes (10): Badge, AddedToShelf, ClickThrough, Copied, HoverPaused, EditTarget, Focus, NewItem (+2 more)

### Community 21 - "Community 21"
Cohesion: 0.18
Nodes (6): elapsed_text(), d2d(), Painter, rect(), span_short(), fade()

### Community 22 - "Community 22"
Cohesion: 0.06
Nodes (11): install(), INSTALL, RELEASE_LEFT, RELEASE_MIDDLE, RELEASE_X, take_chord(), take_menu(), take_outside() (+3 more)

### Community 23 - "Community 23"
Cohesion: 0.13
Nodes (26): centred_buttons(), CORNER_PULL, CORNER_REACH, CORNER_SAMPLES, four_notification_rows_fit_inside_the_default_panel_body(), media_art(), MEDIA_STACKED_MIN_H, media_text_column() (+18 more)

### Community 24 - "Community 24"
Cohesion: 0.14
Nodes (3): FlashRenderer, ImagePart, TextPart

### Community 25 - "Community 25"
Cohesion: 0.09
Nodes (13): AnimConfig, app_config_directory_from(), appdata_override_is_used_before_the_known_folder(), ColorConfig, DEFAULT_FLASH_TEXT, font_migration_changes_only_legacy_defaults_and_round_trips(), MarqueeConfig, new_font_defaults_use_geist() (+5 more)

### Community 26 - "Community 26"
Cohesion: 0.11
Nodes (5): Backdrop, BackdropCache, DEFAULT_DOWNSCALE, MAX_DIM, MIN_DIM

### Community 27 - "Community 27"
Cohesion: 0.10
Nodes (8): apps_use_light_theme(), endpoint(), flip(), KEEP_AWAKE, PERSONALIZE, REFRESH, set_light_theme(), states()

### Community 28 - "Community 28"
Cohesion: 0.10
Nodes (24): SlideKind, .ALL, Calendar, Clipboard, Clock, Controls, Marquee, Media (+16 more)

### Community 29 - "Community 29"
Cohesion: 0.10
Nodes (22): Half, MenuCommand, ClearClipboard, Close, CopyFile, CopyPath, NextPage, Open (+14 more)

### Community 30 - "Community 30"
Cohesion: 0.08
Nodes (5): ICON_PX, reveal(), REVISION, revision(), WAIT_SLICE

### Community 31 - "Community 31"
Cohesion: 0.13
Nodes (17): Content, Layout, Plain, HEIGHT, Mode, Panel, Pill, paint_desktop() (+9 more)

### Community 32 - "Community 32"
Cohesion: 0.16
Nodes (15): TimerConfig, pausing_keeps_the_time_left(), Phase, Focus, LongBreak, ShortBreak, reset(), rounds_lead_to_a_long_break_and_back() (+7 more)

### Community 33 - "Community 33"
Cohesion: 0.11
Nodes (16): ACTIVITY_REVISION, ACTIVITY_TIMEOUT, ActivityPayload, MAX_ACTIVITIES, MAX_DISMISSED_NATIVE_KEYS, MAX_NATIVE_NOTIFICATIONS, MAX_NOTIFICATIONS, NEXT_NOTIF_ID (+8 more)

### Community 35 - "Community 35"
Cohesion: 0.12
Nodes (4): FlashConfig, FlashWindow, exclude_from_capture(), include_in_capture()

### Community 36 - "Community 36"
Cohesion: 0.10
Nodes (20): Hit, Clip, ClipClear, Focus, Media, NotifBack, NotifClear, NotifDismiss (+12 more)

### Community 37 - "Community 37"
Cohesion: 0.09
Nodes (22): runs-on, steps, GALLIUM_DRIVER, LIBGL_ALWAYS_SOFTWARE, VENU_STARTUP_TEST_OPENGL_DIR, jobs, build, name (+14 more)

### Community 38 - "Community 38"
Cohesion: 0.12
Nodes (11): acquire(), AudioMeter, BARS, follow(), key(), REACQUIRE, REST, shape() (+3 more)

### Community 39 - "Community 39"
Cohesion: 0.09
Nodes (21): ADD, CLEAR, CLOSE, COPY, FOLDER, LINK, MAX_H, MENU_W (+13 more)

### Community 40 - "Community 40"
Cohesion: 0.12
Nodes (12): set_toast_duration(), CLICK_THROUGH_ON, NotchManager, OPEN, PENDING, publish(), RESET_CONTROLS, SAVE_DEBOUNCE (+4 more)

### Community 41 - "Community 41"
Cohesion: 0.14
Nodes (10): acquire_single_instance(), create_app_icon_data(), FRAME_MS, IDLE_POLL_MS, main(), run_overlay_thread(), record_event(), install_settings_request_channel() (+2 more)

### Community 42 - "Community 42"
Cohesion: 0.12
Nodes (6): ACCENT_ACRYLIC, ACCENT_POLICY, AccentPolicy, apply(), CompositionData, SetWindowCompositionAttribute

### Community 43 - "Community 43"
Cohesion: 0.20
Nodes (12): BatteryAlertConfig, alert(), low_battery_warns_once_per_level(), NO_BATTERY, plugging_in_and_out_is_announced(), POLL, r(), read() (+4 more)

### Community 44 - "Community 44"
Cohesion: 0.18
Nodes (11): color_picker_button(), ColorMode, Hex, Hsv, Rgb, ColorPickerState, draw_checkerboard(), hex_to_rgba() (+3 more)

### Community 45 - "Community 45"
Cohesion: 0.15
Nodes (16): a_shortcut_needs_a_modifier_and_a_known_key(), HotkeyStatus, Active, InUse, Off, letters_digits_and_function_keys_map_to_their_codes(), MOD_ALT, MOD_CONTROL (+8 more)

### Community 46 - "Community 46"
Cohesion: 0.18
Nodes (8): hsl_to_rgb(), Motion, Ambient, Continuous, Still, MenuColors, premul(), Rgba

### Community 47 - "Community 47"
Cohesion: 0.16
Nodes (7): screenshots_folder(), buf(), pick_files(), pick_folder(), single_and_multiple_picks_both_come_back_as_full_paths(), split_multi(), wide()

### Community 48 - "Community 48"
Cohesion: 0.12
Nodes (17): with, api-token, body_path, files, generate_release_notes, github-artifact-id, if-no-files-found, name (+9 more)

### Community 49 - "Community 49"
Cohesion: 0.17
Nodes (3): FontConfig, CachedText, D2DRenderer

### Community 51 - "Community 51"
Cohesion: 0.22
Nodes (10): clipboard_clear_button(), clipboard_rows(), clipboard_rows_fill_down_then_across(), Rect, shelf_gives_each_section_a_row_and_counts_what_overflows(), shelf_layout(), status_list(), status_list_leaves_a_row_for_adding_until_it_is_full() (+2 more)

### Community 52 - "Community 52"
Cohesion: 0.24
Nodes (4): global_store(), usage_store(), Clock, snapshot()

### Community 53 - "Community 53"
Cohesion: 0.12
Nodes (10): a_fresh_deck_has_the_shelf_but_not_the_clipboard(), CalendarConfig, ClipboardConfig, .MAX, configs_from_before_the_controls_load_with_safe_defaults(), deck_order_lists_every_feature_and_follows_the_deck(), MediaConfig, .AUTO_HEIGHT (+2 more)

### Community 54 - "Community 54"
Cohesion: 0.12
Nodes (14): BorderEffect, .ALL, Aurora, Breathe, Comet, Fire, GlassShine, Heartbeat (+6 more)

### Community 55 - "Community 55"
Cohesion: 0.14
Nodes (15): FlashAnim, .ALL, Fade, SlideLeftToRight, SlideRightToLeft, ZoomCenter, draw_params(), DrawParams (+7 more)

### Community 56 - "Community 56"
Cohesion: 0.13
Nodes (13): lerp(), GUTTER, Palette, PILL_GUTTER, SHADOW_PAD, shadow_spread(), SIZE_BODY, SIZE_CLOCK (+5 more)

### Community 57 - "Community 57"
Cohesion: 0.19
Nodes (3): WallpaperConfig, crop_uv(), preview()

### Community 58 - "Community 58"
Cohesion: 0.13
Nodes (14): CollapsedMode, .ALL, Auto, Calendar, Clock, LastActive, Marquee, Media (+6 more)

### Community 59 - "Community 59"
Cohesion: 0.21
Nodes (11): get_egui_context(), Command, Wake, has_package_identity(), persist_opt_in(), request_access_on_ui_thread(), set_enabled(), set_status() (+3 more)

### Community 60 - "Community 60"
Cohesion: 0.24
Nodes (12): corner(), corners_never_overlap_on_a_short_slab(), cubic_at(), detached_rim_starts_after_its_top_corner(), fused(), fused_rim_runs_tip_to_tip_inside_its_bounds(), Point, Rim (+4 more)

### Community 61 - "Community 61"
Cohesion: 0.15
Nodes (13): MouseTrigger, .ALL, AltClick, BackButton, CtrlClick, ForwardButton, MiddleClick, Off (+5 more)

### Community 62 - "Community 62"
Cohesion: 0.21
Nodes (4): create_app_icon(), debug_log(), request_existing_settings(), SystemTray

### Community 63 - "Community 63"
Cohesion: 0.18
Nodes (9): default_shape_reproduces_the_original_silhouette(), shape_numbers_are_clamped_and_presets_apply(), ShapePreset, Classic, Custom, .PICKS, Pill, Sharp (+1 more)

### Community 64 - "Community 64"
Cohesion: 0.15
Nodes (12): CHORD, CHORD_ACTIVE, L_DOWN, MENU_ACTIVE, MENU_LATCH, OUTSIDE, R_DOWN, R_PENDING (+4 more)

### Community 65 - "Community 65"
Cohesion: 0.17
Nodes (8): assign_sides(), PodKind, Agent, .ALL, Calendar, Media, Timer, seated_pods_keep_their_side()

### Community 66 - "Community 66"
Cohesion: 0.32
Nodes (3): arrow_keys_walk_the_items_and_enter_picks_one(), MenuView, Slot

### Community 68 - "Community 68"
Cohesion: 0.21
Nodes (4): apply_style(), token!, Palette, set()

### Community 69 - "Community 69"
Cohesion: 0.20
Nodes (7): Command, Configure, Refresh, Stop, FolderWatch, run(), sync()

### Community 70 - "Community 70"
Cohesion: 0.26
Nodes (5): Hotkey, InputConfig, .MAX_HOVER_DELAY_MS, reset_controls_restores_behaviour_but_keeps_shortcuts(), the_notch_always_keeps_a_way_to_close()

### Community 72 - "Community 72"
Cohesion: 0.17
Nodes (8): CACHE, States, Toggle, .ALL, DarkMode, KeepAwake, Mic, Sound

### Community 73 - "Community 73"
Cohesion: 0.23
Nodes (6): Edge, Bottom, Left, Right, Top, Win32OverlayWindow

### Community 74 - "Community 74"
Cohesion: 0.27
Nodes (5): EdgeSelection, PaddingConfig, LayoutKey, OverlayManager, RELAYOUT_INTERVAL

### Community 75 - "Community 75"
Cohesion: 0.33
Nodes (6): idle_slot_stays_idle(), settle(), Slot, slot_clears_the_old_occupant_before_the_new_one_grows(), slot_grows_in_and_out(), Slot<K>

### Community 76 - "Community 76"
Cohesion: 0.25
Nodes (6): clean(), ShapeConfig, .MAX_RADIUS, .MAX_SHOULDER, .MAX_STRENGTH, .MIN_FUSED_SHOULDER

### Community 77 - "Community 77"
Cohesion: 0.18
Nodes (9): default_theme(), NotchTheme, Acrylic, .ALL, Blurred, Dark, Frosted, Light (+1 more)

### Community 78 - "Community 78"
Cohesion: 0.29
Nodes (7): clear_all_suppresses_all_current_native_rows_but_not_future_rows(), native(), native_notifications_are_not_filtered_by_webhook_app_rules(), native_snapshot_seeds_silently_and_keeps_only_four_latest(), NativeNotification, operating_system_removal_keeps_local_history_and_dismissal_suppresses_resurrection(), repeated_snapshot_updates_in_place_and_new_event_toasts_once()

### Community 79 - "Community 79"
Cohesion: 0.27
Nodes (4): FlashManager, PREVIEW_REQUESTED, request_preview(), Showing

### Community 80 - "Community 80"
Cohesion: 0.20
Nodes (4): HTCLIENT, HTTRANSPARENT, InputBus, with_input()

### Community 81 - "Community 81"
Cohesion: 0.38
Nodes (3): NotchConfig, .DECK_REVISION, stats_controls_preserve_other_slides_and_the_current_face()

### Community 82 - "Community 82"
Cohesion: 0.20
Nodes (7): NotificationConfig, NotificationGlowStyle, .ALL, CountdownDrain, NeonGlow, RgbBorderMoving, WavyRgbMoving

### Community 83 - "Community 83"
Cohesion: 0.22
Nodes (6): ShelfConfig, .MAX_FOLDER_COUNT, .MAX_ITEMS, the_shelf_pins_each_path_once_and_has_a_ceiling(), Worker, WORKER

### Community 84 - "Community 84"
Cohesion: 0.24
Nodes (9): bounded_text(), compose_toast_text(), empty_and_optional_toast_fields_have_a_compact_fallback(), format_local_time(), long_windows_text_is_limited_before_entering_the_history(), notification_source_key(), notification_text_is_bounded_by_characters(), parse_notification() (+1 more)

### Community 85 - "Community 85"
Cohesion: 0.36
Nodes (8): choosing_runs_once_and_the_card_closes_away(), frame(), items_are_found_where_they_are_drawn(), place(), rows(), size(), the_card_fits_its_rows_and_stays_inside_the_frame(), the_highlight_slides_between_items_but_appears_in_place()

### Community 86 - "Community 86"
Cohesion: 0.28
Nodes (3): font_definitions(), production_font_setup_renders_latin_and_devanagari_fallback(), setup_custom_fonts()

### Community 87 - "Community 87"
Cohesion: 0.22
Nodes (9): concurrency, env, if, needs, runs-on, cancel-in-progress, group, VERSION (+1 more)

### Community 88 - "Community 88"
Cohesion: 0.22
Nodes (5): button_held(), is_trigger(), key_down(), proc(), within()

### Community 89 - "Community 89"
Cohesion: 0.22
Nodes (7): FlashBackground, .ALL, Blur, Dark, Frosted, Transparent, White

### Community 90 - "Community 90"
Cohesion: 0.25
Nodes (3): Notification, NotificationCenter, ToastAlert

### Community 91 - "Community 91"
Cohesion: 0.25
Nodes (7): BOUND_B, BOUND_L, BOUND_R, BOUND_T, MENU_X, MENU_Y, WHEEL

### Community 92 - "Community 92"
Cohesion: 0.39
Nodes (8): steps, steps, id, if, name, run, shell, uses

### Community 93 - "Community 93"
Cohesion: 0.32
Nodes (3): PREVIEW_GAP_SECS, rgba(), show()

### Community 94 - "Community 94"
Cohesion: 0.25
Nodes (5): ScrollPages, Always, ClosedOnly, Off, OpenOnly

### Community 95 - "Community 95"
Cohesion: 0.29
Nodes (6): copy_file(), decode_dib(), drop_files(), Global, put(), wide_bytes()

### Community 96 - "Community 96"
Cohesion: 0.25
Nodes (6): NotificationLevel, Action, Info, Success, Warning, WebhookPayload

### Community 97 - "Community 97"
Cohesion: 0.48
Nodes (5): explicit_settings_flag_wins_when_quiet_flags_are_also_present(), ordinary_launch_starts_quiet_and_settings_flag_opens_window(), settings_visible_from_args(), startup_and_minimized_keep_the_quiet_behavior(), unrelated_arguments_do_not_open_settings()

### Community 98 - "Community 98"
Cohesion: 0.38
Nodes (3): FeatureRow, lerp(), WARNING

### Community 100 - "Community 100"
Cohesion: 0.29
Nodes (5): EffectScope, .ALL, Always, CollapsedOnly, ExpandedOnly

### Community 101 - "Community 101"
Cohesion: 0.29
Nodes (6): FlashLayout, .ALL, TextBottom, TextLeft, TextRight, TextTop

### Community 102 - "Community 102"
Cohesion: 0.29
Nodes (7): AccessState, Allowed, Denied, Error, NeedsConsent, listener_permission_lifecycle_has_explicit_states(), permission_status()

### Community 104 - "Community 104"
Cohesion: 0.38
Nodes (5): upcoming(), clock_text(), day_word(), event_time_range(), span_text()

### Community 105 - "Community 105"
Cohesion: 0.33
Nodes (5): ACTIVITIES, Activity, current_activity(), current_elapsed_secs(), has_activity()

### Community 106 - "Community 106"
Cohesion: 0.33
Nodes (4): detach_listener(), newest_notification_indices(), refresh_snapshot(), snapshot_selection_is_timestamp_ordered_and_memory_bounded()

### Community 107 - "Community 107"
Cohesion: 0.33
Nodes (5): jobs, name, permissions, actions, contents

### Community 108 - "Community 108"
Cohesion: 0.33
Nodes (6): on, push, workflow_dispatch, branches, paths, tags

### Community 109 - "Community 109"
Cohesion: 0.47
Nodes (5): bounded(), history_declined(), private(), read_text(), read_wide()

### Community 110 - "Community 110"
Cohesion: 0.33
Nodes (5): FlashContent, .ALL, Both, Image, Text

### Community 111 - "Community 111"
Cohesion: 0.33
Nodes (4): OpenWith, Both, Click, Hover

### Community 114 - "Community 114"
Cohesion: 0.47
Nodes (4): current_time_formatted(), handle_webhook_client(), start_activity(), stop_activity()

### Community 115 - "Community 115"
Cohesion: 0.40
Nodes (5): extract_icon(), Icon, normalise_alpha(), opaque_thumbnails_get_alpha_and_icons_get_premultiplied(), read_bitmap()

### Community 116 - "Community 116"
Cohesion: 0.40
Nodes (4): ShelfEntry, Snapshot, snapshot_cell(), SNAPSHOT

### Community 117 - "Community 117"
Cohesion: 0.40
Nodes (5): SIGNPATH_API_TOKEN, SIGNPATH_ORGANIZATION_ID, SIGNPATH_PROJECT_SLUG, SIGNPATH_SIGNING_POLICY_SLUG, env

### Community 118 - "Community 118"
Cohesion: 0.40
Nodes (5): resolve-version, should-release, version, outputs, runs-on

### Community 119 - "Community 119"
Cohesion: 0.40
Nodes (4): display_name(), hidden(), names_drop_shortcut_and_program_extensions_only(), newest_files()

### Community 121 - "Community 121"
Cohesion: 0.50
Nodes (4): NotchAlign, Center, Left, Right

### Community 122 - "Community 122"
Cohesion: 0.50
Nodes (3): history_keeps_newest_first_without_repeats(), items(), remember()

### Community 127 - "Community 127"
Cohesion: 1.00
Nodes (3): override_or(), slide_collapsed_size(), slide_expanded_size()

## Knowledge Gaps
- **211 isolated node(s):** `Close`, `Minimize`, `Pin`, `Search`, `Notch` (+206 more)
  These have ≤1 connection - possible missing edges. (Counts symbols only; 872 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **28 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `AppConfig` connect `Community 16` to `Community 0`, `Community 4`, `Community 5`, `Community 6`, `Community 8`, `Community 9`, `Community 10`, `Community 11`, `Community 13`, `Community 17`, `Community 19`, `Community 20`, `Community 21`, `Community 23`, `Community 25`, `Community 28`, `Community 31`, `Community 35`, `Community 40`, `Community 41`, `Community 46`, `Community 49`, `Community 50`, `Community 52`, `Community 57`, `Community 59`, `Community 71`, `Community 73`, `Community 74`, `Community 79`, `Community 81`, `Community 98`, `Community 99`, `Community 104`, `Community 123`, `Community 127`?**
  _High betweenness centrality (0.229) - this node is a cross-community bridge._
- **Are the 2 inferred relationships involving `SettingsApp` (e.g. with `App` and `PreviewCache`) actually correct?**
  _`SettingsApp` has 2 INFERRED edges - model-reasoned connections that need verification._
- **What connects `Close`, `Minimize`, `Pin` to the rest of the system?**
  _211 weakly-connected nodes found - possible missing edges._
- **Should `Community 0` be split into smaller, more focused modules?**
  _Cohesion score 0.03413654618473896 - nodes in this community are weakly interconnected._
- **Why does `SlideKind` connect `Community 28` to `Community 0`, `Community 5`, `Community 6`, `Community 9`, `Community 10`, `Community 11`, `Community 12`, `Community 16`, `Community 17`, `Community 20`, `Community 21`, `Community 25`, `Community 31`, `Community 50`, `Community 52`, `Community 53`, `Community 65`, `Community 81`, `Community 98`, `Community 127`?**
  _High betweenness centrality (0.071) - this node is a cross-community bridge._
- **Are the 8 inferred relationships involving `Painter` (e.g. with `draw_checkerboard()` and `paint_glyph()`) actually correct?**
  _`Painter` has 8 INFERRED edges - model-reasoned connections that need verification._
- **Should `Community 1` be split into smaller, more focused modules?**
  _Cohesion score 0.053994732221246705 - nodes in this community are weakly interconnected._