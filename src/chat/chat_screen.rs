use std::cell::RefCell;
use std::rc::Rc;

use makepad_widgets::*;
use moly_kit::*;
use moly_kit::utils::asynchronous::spawn;
use moly_protocol::open_ai::Role;

use crate::data::chats::chat::ChatMessage;
use crate::data::chats::chat_entity::ChatEntityId;
use crate::data::providers::ProviderType;
use crate::data::store::Store;
use crate::shared::actions::ChatAction;

use super::model_selector_item::ModelSelectorAction;

const OPEN_AI_KEY: Option<&str> = option_env!("OPEN_AI_KEY");
const OPEN_ROUTER_KEY: Option<&str> = option_env!("OPEN_ROUTER_KEY");
const SILICON_FLOW_KEY: Option<&str> = option_env!("SILICON_FLOW_KEY");

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::chat::chat_panel::ChatPanel;
    use crate::chat::chat_history::ChatHistory;
    use crate::chat::chat_params::ChatParams;
    use crate::chat::model_selector::ModelSelector;
    use moly_kit::widgets::chat::Chat;

    pub ChatScreen = {{ChatScreen}} {
        width: Fill,
        height: Fill,
        spacing: 10,

        <View> {
            width: Fit,
            height: Fill,

            chat_history = <ChatHistory> {}
        }

        <View> {
            width: Fill,
            height: Fill,
            align: {x: 0.5},
            padding: {top: 48, bottom: 48 }
            flow: Down,

            model_selector = <ModelSelector> {}
            chat = <Chat> {}
            // chat_panel = <ChatPanel> {}
        }

        // TODO: Add chat params back in, only when the model is a local model (MolyServer)
        // currenlty MolyKit does not support chat params
        // 
        // <View> {
        //     width: Fit,
        //     height: Fill,
        // 
        //     chat_params = <ChatParams> {}
        // }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ChatScreen {
    #[deref]
    view: View,

    #[rust(true)]
    first_render: bool,

    #[rust]
    should_load_repo_to_store: bool,

    #[rust]
    creating_bot_repo: bool,

    #[rust]
    message_container: Rc<RefCell<Option<Message>>>,
}

impl Widget for ChatScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        let chat = self.chat(id!(chat));
        self.ui_runner().handle(cx, event, scope, self);
        self.widget_match_event(cx, event, scope);
  
        // TODO This check is actually copied from Makepad view.rs file
        // It's not clear why it's needed here, but without this line
        // the "View all files" link in Discover section does not work after visiting the chat screen
        if self.visible || !event.requires_visibility() {
            self.view.handle_event(cx, event, scope);
        }

        let store = scope.data.get_mut::<Store>().unwrap();

        // TODO(MolyKit): Cleanup, might be unnecessary to track first_render
        let should_recreate_bot_repo = store.bot_repo.is_none();

        if self.should_load_repo_to_store {
            println!("loading repo to store");
            store.bot_repo = self.chat(id!(chat)).read().bot_repo.clone();
            self.should_load_repo_to_store = false;
            // TODO(MolyKit): Cleanup, might be unnecessary to redraw all
            cx.redraw_all();
        } else if (self.first_render || should_recreate_bot_repo) && !self.creating_bot_repo {
            self.create_bot_repo(cx, scope);
            self.first_render = false;
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {

        let store = scope.data.get_mut::<Store>().unwrap();
        let current_chat = store.chats.get_current_chat();

        // TODO(Julian): next step, load the messages from the current chat into the chat widget
        // Load the messages from the current chat into the chat widget
        // if let Some(chat) = current_chat {
        //     let messages = chat.borrow().messages.clone();
        //     let messages_widget_ref = self.messages(id!(chat.messages)).write();
        //     messages_widget_ref.messages = messages;
        // }

        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for ChatScreen {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let store = scope.data.get_mut::<Store>().unwrap();
        let mut chat_widget = self.chat(id!(chat));
        for action in actions {
            // Handle model selector actions
            match action.cast() {
                ModelSelectorAction::ModelSelected(downloaded_file) => {
                // TODO(MolyKit): Handle local files
                // store.load_model(&downloaded_file.file);
                // let bot_id = BotId::from(remote_model.id.0.as_str());
                // chat_widget.write().bot_id = Some(bot_id);
                // self.focus_on_prompt_input_pending = true;
                }
                ModelSelectorAction::AgentSelected(agent) => {
                    // TODO(MolyKit): Handle agents
                    // let bot_id = BotId::from(remote_model.id.0.as_str());
                    // chat_widget.write().bot_id = Some(bot_id);
                    // self.focus_on_prompt_input_pending = true;
                }
                ModelSelectorAction::RemoteModelSelected(remote_model) => {
                    let bot_id = BotId::from(remote_model.id.0.as_str());
                    chat_widget.write().bot_id = Some(bot_id);

                    if let Some(chat) = store.chats.get_current_chat() {
                        chat.borrow_mut().associated_entity = Some(ChatEntityId::RemoteModel(remote_model.id.clone()));
                        chat.borrow().save();
                    }

                    // self.focus_on_prompt_input_pending = true;
                }
                _ => {}
            }

            // Handle chat start
            match action.cast() {
                // This action is dispatched whenever an entity (like an agent) is clicked on the chat history
                ChatAction::Start(handler) => match handler {
                    ChatEntityId::ModelFile(file_id) => {
                        if let Some(file) = store.downloads.get_file(&file_id) {
                            store.chats.create_empty_chat_and_load_file(file);
                            // self.focus_on_prompt_input_pending = true;
                        }
                    }
                    ChatEntityId::Agent(agent_id) => {
                        store.chats.create_empty_chat_with_agent(&agent_id);
                        // self.focus_on_prompt_input_pending = true;
                    },
                    ChatEntityId::RemoteModel(model_id) => {
                        println!("creating empty chat with remote model: {:?}", model_id);
                        store.chats.create_empty_chat_with_remote_model(&model_id);
                        // self.focus_on_prompt_input_pending = true;
                    }
                },
                _ => {}
            }

            // let mut new_message_to_insert: Option<Message> = None;

            // let message_container = Rc::new(RefCell::new(None)); 
            let message_ref = Rc::clone(&self.message_container);

            // TODO(MolyKit): Hook into new messages to update history
            self.chat(id!(chat)).write_with(|chat| {
                let message_ref = Rc::clone(&message_ref);
                chat.set_hook_after(move |group, _, _| {
                    for task in group.iter() {
                        if let ChatTask::InsertMessage(index, message) = task {
                            log!("After hook ChatTask::InsertMessage");
                            message_ref.borrow_mut().replace(message.clone());
                            // new_message_to_insert = Some(message.clone());

                            // if let Some(current_chat) = store.chats.get_current_chat() {
                                // let next_id = current_chat.borrow().messages.last().map(|m| m.id).unwrap_or(0) + 1;
                                // current_chat.borrow_mut().messages.push(ChatMessage {
                                //     id: next_id,
                                //     role: Role::User,
                                //     username: None,
                                //     entity: None,
                                //     content: message.body.clone(),
                                //     articles: vec![],
                                //     stages: vec![],
                                // });
                
                                // current_chat.borrow_mut().update_title_based_on_first_message();
                                // current_chat.borrow().save();
                                // println!("Updated title based on first message");
                            // }
                        }
                    }
                });
            });

            if let Some(new_message) = self.message_container.borrow_mut().take() {
                println!("new message to insert");
                if let Some(current_chat) = store.chats.get_current_chat() {
                    let next_id = current_chat.borrow().messages.last().map(|m| m.id).unwrap_or(0) + 1;
                    current_chat.borrow_mut().messages.push(ChatMessage {
                        id: next_id,
                        role: Role::User,
                        username: None,
                        entity: None,
                        content: new_message.body.clone(),
                        articles: vec![],
                        stages: vec![],
                    });

                    current_chat.borrow_mut().update_title_based_on_first_message();
                    current_chat.borrow().save();
                    println!("updated title based on first message");
                    // TODO(Julian) the chat hisotry card does not get immediately redrawn with the new title.
                    self.redraw(cx);
                }
            }; // TODO(Julian) this semicolon is needed to end the closure, find a better way to do this

            // TODO(Julian) whenever a chat history card is clicked, load those messages into the chat widget

            // Handle chat line actions
            // TODO(MolyKit): handle these by hookinng into MolyKit's Chat widget
            // match action.cast() {
            //     ChatLineAction::Delete(id) => {
            //         store.chats.delete_chat_message(id);
            //         self.redraw(cx);
            //     }
            //     ChatLineAction::Edit(id, updated, regenerate) => {
            //         if regenerate {
            //             self.send_message(cx, scope, updated, Some(id));
            //             return;
            //         } else {
            //             store.edit_chat_message(id, updated);
            //         }
            //         self.redraw(cx);
            //     }
            //     _ => {}
            // }
        }
    }
}


impl ChatScreen {
    fn create_bot_repo(&mut self, cx: &mut Cx, scope: &mut Scope) {
        let store = scope.data.get_mut::<Store>().unwrap();

        let multi_client = {
            // let moly = OpenAIClient::new("http://localhost:8085".into());
            // let ollama = OpenAIClient::new("http://localhost:11434".into());
    
            let mut multi_client = MultiClient::new();
            // client.add_client(Box::new(moly));
            // client.add_client(Box::new(ollama));
    
            // Only add OpenAI client if API key is present
            // if let Some(key) = OPEN_AI_KEY {
            //     let openai_url = "https://api.openai.com";
            //     let mut openai = OpenAIClient::new(openai_url.into());
            //     openai.set_key(key);
            //     client.add_client(Box::new(openai));
            // }
    
            // Only add OpenRouter client if API key is present
            // if let Some(key) = OPEN_ROUTER_KEY {
            //     let open_router_url = "https://openrouter.ai/api";
            //     let mut open_router = OpenAIClient::new(open_router_url.into());
            //     open_router.set_key(key);
            //     multi_client.add_client(Box::new(open_router));
            // }
    
            // // Only add SiliconFlow client if API key is present
            // if let Some(key) = SILICON_FLOW_KEY {
            //     let siliconflow_url = "https://api.siliconflow.cn";
            //     let mut siliconflow = OpenAIClient::new(siliconflow_url.into());
            //     siliconflow.set_key(key);
            //     client.add_client(Box::new(siliconflow));
            // }

            for provider in store.chats.providers.iter() {

                match provider.1.provider_type {
                    ProviderType::OpenAI => {
                        if provider.1.enabled && provider.1.api_key.is_some() {
                            let mut new_client = OpenAIClient::new(provider.1.url.clone());
                            if let Some(key) = provider.1.api_key.as_ref() {
                                println!("Setting key for client: {}", key);
                                new_client.set_key(&key);
                            }
                            multi_client.add_client(Box::new(new_client));
                        }
                    },
                    // TODO(MolyKit) add support for other clients here
                    ProviderType::MoFa => {
                        // For MoFa we don't require an API key
                        if provider.1.enabled {
                            let mut new_client = OpenAIClient::new(provider.1.url.clone());
                            if let Some(key) = provider.1.api_key.as_ref() {
                                println!("Setting key for client: {}", key);
                                new_client.set_key(&key);
                            }
                            multi_client.add_client(Box::new(new_client));
                            // multi_client.add_client(Box::new(MoFaClient::new(provider.1.url.clone())));
                        }
                    },
                    ProviderType::DeepInquire => {
                        // TODO
                    }
                }
            }

            multi_client
        };
    
        let mut repo: BotRepo = multi_client.into();
        self.chat(id!(chat)).write().bot_repo = Some(repo.clone());

        self.creating_bot_repo = true;

        let ui = self.ui_runner();
            spawn(async move {
                repo.load().await.expect("TODO: Handle loading better");
                ui.defer_with_redraw(move |me, _cx, _scope| {
                // println!("bots: {:?}", repo.bots());
                    
                // me.chat(id!(chat)).write().bot_id = Some("openai/gpt-4o-https://openrouter.ai/api".into());
                me.should_load_repo_to_store = true;
                me.creating_bot_repo = false;
                // me.fill_selector(repo.bots());
                // me.chat(id!(chat)).write().visible = true;
            });
        });
    }
}
