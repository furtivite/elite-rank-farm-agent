use agent_core::{lang, set_lang, tr, Lang};

#[test]
fn messages_follow_the_chosen_language() {
    let count = 3;
    assert_eq!(lang(), Lang::En, "по умолчанию — английский");
    assert_eq!(tr!("sent: {count}", "отправлено: {count}"), "sent: 3");

    set_lang(Lang::Ru);
    assert_eq!(tr!("sent: {count}", "отправлено: {count}"), "отправлено: 3");

    set_lang(Lang::En);
    assert_eq!(tr!("sent: {}", "отправлено: {}", count), "sent: 3");
}
