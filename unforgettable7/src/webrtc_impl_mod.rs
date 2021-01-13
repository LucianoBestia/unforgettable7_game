// webrtc_impl_mod.rs
//! specific implementation of WebRTC communication

// region: use
use crate::*;
//for trait rrc.render_template
use rust_wasm_dodrio_templating::html_template_mod::HtmlTemplating;
use rust_wasm_webrtc::webrtcmod::{WebRtcTrait, ChatMessage};

use unwrap::unwrap;
use wasm_bindgen::{JsCast};
//use wasm_bindgen_futures::spawn_local;
use dodrio::{RenderContext, Node, RootRender};
use web_sys::{Event, WebSocket, KeyboardEvent, RtcPeerConnection, RtcDataChannel};
// endregion

/// game data
pub struct WebRtcData {
    /// web socket communication between players - cloned
    pub rtc_ws: Option<WebSocket>,
    /// webrtc connection
    pub rtc_peer_connection: Option<RtcPeerConnection>,
    /// rtc data channel
    pub rtc_data_channel: Option<RtcDataChannel>,
    /// my ws uid
    pub rtc_my_ws_uid: usize,
    /// receiver for webrtc
    pub rtc_receiver_ws_uid: usize,
    /// accepted call
    pub rtc_accepted_call: bool,
    /// queue for ice candidate
    pub rtc_ice_queue: Vec<String>,
    /// chat messages
    pub rtc_chat: Vec<ChatMessage>,
    /// rtc_is_data_channel_open
    pub rtc_is_data_channel_open: bool,
    /// if a render event comes while we are typing in input
    /// we will loose the content. So on every onkeyup, I have to store it in the struct.
    pub rtc_my_message: String,
}

impl WebRtcData {
    /// constructor
    pub fn new(my_ws_uid: usize) -> Self {
        // return from constructor
        WebRtcData {
            rtc_ws: None,
            rtc_peer_connection: None,
            rtc_data_channel: None,
            rtc_accepted_call: false,
            rtc_ice_queue: vec![],
            rtc_my_ws_uid: my_ws_uid,
            rtc_receiver_ws_uid: 0,
            rtc_chat: vec![],
            rtc_is_data_channel_open: false,
            rtc_my_message: "".to_string(),
        }
    }
    /// send msg over ws
    pub fn send_webrtc_msg_to_receiver(&self, msg_data: &WsMessageGameData) {
        // send single msg to every receiver (except self)
        Self::send_webrtc_msg_to_single_receiver(&self, self.rtc_receiver_ws_uid, msg_data)
    }

    /// send msg over ws to single receiver
    pub fn send_webrtc_msg_to_single_receiver(
        &self,
        msg_receiver_ws_uid: usize,
        msg_data: &WsMessageGameData,
    ) {
        if msg_receiver_ws_uid != self.rtc_my_ws_uid {
            let ws_message = websocket_boiler_mod::WsMessageForReceiver {
                msg_sender_ws_uid: self.rtc_my_ws_uid,
                msg_receiver_ws_uid,
                msg_data: msg_data.clone(),
            };
            let json_message = unwrap!(serde_json::to_string(&ws_message));
            use rust_wasm_websocket::websocketmod::WebSocketTrait;
            websocket_boiler_mod::WebSocketData::ws_send_json_msg_with_retry(
                unwrap!(self.rtc_ws.as_ref()),
                json_message,
            );
        }
    }
}

impl WebRtcTrait for WebRtcData {
    // region: getter setter
    fn get_rtc_ws(&self) -> &WebSocket {
        unwrap!(self.rtc_ws.as_ref())
    }
    fn set_rtc_ws(&mut self, ws: WebSocket) {
        self.rtc_ws = Some(ws);
    }
    fn get_rtc_my_ws_uid(&self) -> usize {
        self.rtc_my_ws_uid
    }
    fn get_rtc_receiver_ws_uid(&self) -> usize {
        self.rtc_receiver_ws_uid
    }
    fn set_rtc_receiver_ws_uid(&mut self, ws_uid: usize) {
        self.rtc_receiver_ws_uid = ws_uid;
    }
    fn get_rtc_peer_connection(&self) -> RtcPeerConnection {
        self.rtc_peer_connection.as_ref().unwrap().clone()
    }
    fn set_rtc_peer_connection(&mut self, rpc: RtcPeerConnection) {
        self.rtc_peer_connection = Some(rpc);
    }
    fn set_rtc_data_channel(&mut self, channel: RtcDataChannel) {
        self.rtc_data_channel = Some(channel);
    }
    fn set_rtc_is_data_channel_open(&mut self, is_open: bool) {
        self.rtc_is_data_channel_open = is_open;
    }
    fn get_rtc_accepted_call(&self) -> bool {
        self.rtc_accepted_call
    }
    fn set_rtc_accepted_call(&mut self, accepted: bool) {
        self.rtc_accepted_call = accepted;
    }
    fn get_mut_rtc_chat(&mut self) -> &mut Vec<ChatMessage> {
        &mut self.rtc_chat
    }
    fn get_rtc_ice_queue(&self) -> &Vec<String> {
        &self.rtc_ice_queue
    }
    fn get_mut_rtc_ice_queue(&mut self) -> &mut Vec<String> {
        &mut self.rtc_ice_queue
    }
    fn get_rtc_data_channel(&self) -> &RtcDataChannel {
        unwrap!(self.rtc_data_channel.as_ref())
    }
    // endregion: getter setter
    fn get_web_rtc_data_from_root_render(root: &mut dyn RootRender) -> &mut WebRtcData {
        let rrc = root.unwrap_mut::<RootRenderingComponent>();
        //return
        &mut rrc.web_data.web_rtc_data
    }
    /// send offer over websocket to establish peer connection
    fn web_rtc_send_offer(&mut self, sdp: String) {
        //websysmod::debug_write("web_rtc_send_offer()");
        let msg_data = game_data_mod::WsMessageGameData::MsgWebrtcOffer { sdp };
        self.send_webrtc_msg_to_receiver(&msg_data);
    }
    /// send answer over websocket to establish peer connection
    fn web_rtc_send_answer(&self, sdp: String) {
        //websysmod::debug_write("web_rtc_send_answer()");
        let msg_data = game_data_mod::WsMessageGameData::MsgWebrtcAnswer { sdp: sdp };
        self.send_webrtc_msg_to_receiver(&msg_data);
    }
    /// send offer over websocket to establish peer connection
    fn web_rtc_send_ice_candidates(&mut self) {
        for sdp in self.get_rtc_ice_queue() {
            //websysmod::debug_write("web_rtc_send_ice_candidate()");
            let sdp = sdp.to_string();
            let msg_data = game_data_mod::WsMessageGameData::MsgWebrtcIceCandidate { sdp };
            self.send_webrtc_msg_to_receiver(&msg_data);
        }
        self.get_mut_rtc_ice_queue().truncate(0);
    }
}

/// on key up only for Enter
pub fn web_rtc_receiver_ws_uid_onkeyup(
    vdom: dodrio::VdomWeak,
    rrc: &mut RootRenderingComponent,
    event: Event,
) {
    let keyboard_event = event.dyn_into::<KeyboardEvent>();
    //websysmod::debug_write(&format!("web_rtc_receiver_ws_uid_onkeyup: {:?}",&keyboard_event));
    if let Ok(keyboard_event) = keyboard_event {
        //websysmod::debug_write(&keyboard_event.key());
        if keyboard_event.key() == "Enter" {
            // same as button click
            rrc.web_data
                .web_rtc_data
                .web_rtc_start(vdom, unwrap!(rrc.web_data.websocket_data.ws.clone()));
        }
    }
}

/// on key up only for Enter
pub fn web_rtc_chat_text_onkeyup(
    vdom: dodrio::VdomWeak,
    rrc: &mut RootRenderingComponent,
    event: Event,
) {
    let keyboard_event = event.dyn_into::<KeyboardEvent>();
    //websysmod::debug_write(&format!("on key up: {:?}",&keyboard_event));
    if let Ok(keyboard_event) = keyboard_event {
        // websysmod::debug_write(&keyboard_event.key());
        if keyboard_event.key() == "Enter" {
            // same as button click
            rrc.web_data.web_rtc_data.web_rtc_send_chat(vdom);
        } else {
            rrc.web_data.web_rtc_data.rtc_my_message =
                websysmod::get_input_element_value_string_by_id("web_rtc_chat_text");
        }
    }
}

/// render messages
pub fn web_rtc_div_messages<'a>(
    rrc: &RootRenderingComponent,
    cx: &mut RenderContext<'a>,
) -> Vec<Node<'a>> {
    let mut vec_nodes = Vec::<Node>::new();

    let mut index = rrc.web_data.web_rtc_data.rtc_chat.len();
    // reverse a vector old school
    if index > 0 {
        index -= 1;
        loop {
            let chat_msg = &rrc.web_data.web_rtc_data.rtc_chat[index];
            let template_name = format!("message_sender{}", chat_msg.sender);
            let mut html_template = rrc.web_data.get_sub_template(&template_name);
            html_template = html_template.replace("replace_in_code_with_msg", &chat_msg.msg);
            let node = unwrap!(rrc.render_template(
                cx,
                &html_template,
                rust_wasm_dodrio_templating::html_template_mod::HtmlOrSvg::Html
            ));
            vec_nodes.push(node);
            if index == 0 {
                break;
            } else {
                index -= 1;
            }
        }
    }
    //return
    vec_nodes
}
