use std::rc::Rc;
use std::ops::Fn;
use dioxus::prelude::*;

pub static CONFIRM_MODAL: GlobalSignal<ConfirmModalState> =
    GlobalSignal::new(|| ConfirmModalState::default());

#[derive(Default)]
pub struct ConfirmModalState {
    pub open: bool,
    pub message: String,
    pub on_yes: Option<Rc<dyn Fn()>>,
    pub on_no: Option<Rc<dyn Fn()>>,
}

impl ConfirmModalState {
    pub fn open(
        &mut self,
        message: impl Into<String>,
        on_yes: Option<Rc<dyn Fn()>>,
        on_no: Option<Rc<dyn Fn()>>,
    ) {
        self.message = message.into();
        self.on_yes = on_yes;
        self.on_no = on_no;
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
    }
}