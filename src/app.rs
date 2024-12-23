use crate::chat::chat_panel::ChatPanelAction;
use crate::data::downloads::download::DownloadFileAction;
use crate::data::downloads::DownloadPendingNotification;
use crate::data::store::*;
use crate::landing::model_files_item::ModelFileItemAction;
use crate::shared::actions::{ChatAction, DownloadAction};
use crate::shared::download_notification_popup::{
    DownloadNotificationPopupAction, DownloadNotificationPopupWidgetRefExt, DownloadResult,
    PopupAction,
};
use crate::shared::popup_notification::PopupNotificationWidgetRefExt;
use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import makepad_draw::shader::std::*;

    import crate::shared::styles::*;
    import crate::shared::widgets::*;
    import crate::shared::popup_notification::*;
    import crate::shared::widgets::SidebarMenuButton;
    import crate::shared::download_notification_popup::DownloadNotificationPopup;
    import crate::shared::desktop_buttons::MolyDesktopButton;

    import crate::landing::landing_screen::LandingScreen;
    import crate::landing::model_card::ModelCardViewAllModal;
    import crate::chat::chat_screen::ChatScreen;
    import crate::my_models::my_models_screen::MyModelsScreen;
    import crate::settings::settings_screen::SettingsScreen;


    ICON_DISCOVER = dep("crate://self/resources/icons/discover.svg")
    ICON_CHAT = dep("crate://self/resources/icons/chat.svg")
    ICON_MY_MODELS = dep("crate://self/resources/icons/my_models.svg")
    ICON_SETTINGS = dep("crate://self/resources/icons/settings.svg")

    App = {{App}} {
        ui: <Window> {
            window: {inner_size: vec2(1440, 1024)},
            pass: {clear_color: #fff}

            caption_bar = {
                caption_label = <View> {} // empty view to remove the default caption label
                windows_buttons = <View> {
                    visible: false,
                    width: Fit, height: Fit,
                    min = <MolyDesktopButton> {draw_bg: {button_type: WindowsMin}}
                    max = <MolyDesktopButton> {draw_bg: {button_type: WindowsMax}}
                    close = <MolyDesktopButton> {draw_bg: {button_type: WindowsClose}}
                }
            }

            body = {
                flow: Overlay
                width: Fill,
                height: Fill,

                root = <View> {
                    width: Fill,
                    height: Fill,

                    sidebar_menu = <RoundedView> {
                        width: 100,
                        height: Fill,
                        flow: Down, spacing: 20.0,
                        padding: { top: 80, bottom: 20 },

                        align: {x: 0.5, y: 0.0},

                        show_bg: true,
                        draw_bg: {
                            color: (SIDEBAR_BG_COLOR),
                            instance radius: 0.0,
                            border_color: #EAECF0,
                            border_width: 1.2,
                        }

                        discover_tab = <SidebarMenuButton> {
                            animator: {selected = {default: on}}
                            text: "Discover",
                            draw_icon: {
                                svg_file: (ICON_DISCOVER),
                            }
                        }
                        chat_tab = <SidebarMenuButton> {
                            text: "Chat",
                            draw_icon: {
                                svg_file: (ICON_CHAT),
                            }
                        }
                        my_models_tab = <SidebarMenuButton> {
                            text: "My Models",
                            draw_icon: {
                                svg_file: (ICON_MY_MODELS),
                            }
                        }
                        <HorizontalFiller> {}
                        settings_tab = <SidebarMenuButton> {
                            text: "Settings",
                            draw_icon: {
                                svg_file: (ICON_SETTINGS),
                            }
                        }
                    }

                    application_pages = <View> {
                        margin: 0.0,
                        padding: 0.0,

                        flow: Overlay,

                        width: Fill,
                        height: Fill,

                        discover_frame = <LandingScreen> {visible: true}
                        chat_frame = <ChatScreen> {visible: false}
                        my_models_frame = <MyModelsScreen> {visible: false}
                        settings_frame = <SettingsScreen> {visible: false}
                    }
                }

                popup_notification = <PopupNotification> {
                    content: {
                        popup_download_notification = <DownloadNotificationPopup> {}
                    }
                }
            }
        }
    }
}

app_main!(App);

#[derive(Live, LiveHook)]
pub struct App {
    #[live]
    ui: WidgetRef,

    #[rust]
    store: Store,
}

impl LiveRegister for App {
    fn live_register(cx: &mut Cx) {
        makepad_widgets::live_design(cx);
        makepad_code_editor::live_design(cx);

        crate::shared::live_design(cx);
        crate::landing::live_design(cx);
        crate::chat::live_design(cx);
        crate::my_models::live_design(cx);
        crate::settings::live_design(cx);
    }
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        let scope = &mut Scope::with_data(&mut self.store);
        self.ui.handle_event(cx, event, scope);
        self.match_event(cx, event);
    }
}

impl MatchEvent for App {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        self.ui
            .radio_button_set(ids!(
                sidebar_menu.discover_tab,
                sidebar_menu.chat_tab,
                sidebar_menu.my_models_tab,
                sidebar_menu.settings_tab,
            ))
            .selected_to_visible(
                cx,
                &self.ui,
                actions,
                ids!(
                    application_pages.discover_frame,
                    application_pages.chat_frame,
                    application_pages.my_models_frame,
                    application_pages.settings_frame,
                ),
            );

        for action in actions.iter() {
            self.store.handle_action(action);

            if let Some(_) = action.downcast_ref::<DownloadFileAction>() {
                self.notify_downloaded_files(cx);
            }

            match action.cast() {
                StoreAction::Search(keywords) => {
                    self.store.search.load_search_results(keywords);
                }
                StoreAction::ResetSearch => {
                    self.store.search.load_featured_models();
                }
                StoreAction::Sort(criteria) => {
                    self.store.search.sort_models(criteria);
                }
                _ => {}
            }

            match action.cast() {
                ModelFileItemAction::Download(file_id) => {
                    let (model, file) = self.store.get_model_and_file_download(&file_id);
                    self.store.downloads.download_file(model, file);
                    self.ui.redraw(cx);
                }
                _ => {}
            }

            match action.cast() {
                DownloadAction::Play(file_id) => {
                    let (model, file) = self.store.get_model_and_file_download(&file_id);
                    self.store.downloads.download_file(model, file);
                    self.ui.redraw(cx);
                }
                DownloadAction::Pause(file_id) => {
                    self.store.downloads.pause_download_file(&file_id);
                    self.ui.redraw(cx);
                }
                DownloadAction::Cancel(file_id) => {
                    self.store.downloads.cancel_download_file(&file_id);
                    self.ui.redraw(cx);
                }
                _ => {}
            }

            if let ChatAction::Start(_) = action.cast() {
                let chat_radio_button = self.ui.radio_button(id!(chat_tab));
                chat_radio_button.select(cx, &mut Scope::empty());
            }

            if let PopupAction::NavigateToMyModels = action.cast() {
                let my_models_radio_button = self.ui.radio_button(id!(my_models_tab));
                my_models_radio_button.select(cx, &mut Scope::empty());
            }

            if let ChatPanelAction::NavigateToDiscover = action.cast() {
                let discover_radio_button = self.ui.radio_button(id!(discover_tab));
                discover_radio_button.select(cx, &mut Scope::empty());
            }

            if matches!(
                action.cast(),
                DownloadNotificationPopupAction::ActionLinkClicked
                    | DownloadNotificationPopupAction::CloseButtonClicked
            ) {
                self.ui
                    .popup_notification(id!(popup_notification))
                    .close(cx);
            }
        }
    }
}

impl App {
    fn notify_downloaded_files(&mut self, cx: &mut Cx) {
        if let Some(notification) = self.store.downloads.next_download_notification() {
            let mut popup = self
                .ui
                .download_notification_popup(id!(popup_download_notification));

            match notification {
                DownloadPendingNotification::DownloadedFile(file) => {
                    popup.set_data(&file, DownloadResult::Success);
                }
                DownloadPendingNotification::DownloadErrored(file) => {
                    popup.set_data(&file, DownloadResult::Failure);
                }
            }

            self.ui.popup_notification(id!(popup_notification)).open(cx);
        }
    }
}
