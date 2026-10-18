use agent_sync::{SendError, SendPace};

use crate::say_tr;

pub(super) fn learn_pace(pace: &mut SendPace, error: &SendError, what: &str) {
    let SendError::Retryable {
        status: 429,
        retry_after,
        ..
    } = error
    else {
        return;
    };

    let before = pace.interval();
    pace.learn(*retry_after);
    if pace.interval() != before {
        say_tr!(
            "  the server asks for less often: {what} interval {} → {} s",
            "  сервер просит реже: интервал {what} {} → {} с",
            before.as_secs(),
            pace.interval().as_secs(),
        );
    }
}
