use lithemux_app::terminal_view::{GridPoint, GridSize, KeyModifiers, KeyPress, SelectionKind};

use crate::support::{SIZE, start, wait_for};

// PLAN §1.1: the process runs on a PTY and its output lands on the grid.
#[test]
fn what_the_program_prints_shows_on_the_screen() {
    let session = start("greet");
    wait_for(&session, "ready");
}

// PLAN §1.1
#[test]
fn what_is_typed_reaches_the_program() {
    let session = start("echo-lines");
    wait_for(&session, "ready");
    session.write("año ¿qué?\r".as_bytes().to_vec());
    wait_for(&session, "got:año ¿qué?");
}

// PLAN §1.2 test 1: the program is told it can use 256 colors and truecolor.
#[test]
fn the_program_sees_a_color_terminal() {
    let session = start("environment");
    wait_for(&session, "term:xterm-256color color:truecolor");
}

// PLAN §1.2 test 5: the program learns the new size (SIGWINCH and the PTY size).
#[test]
fn a_resize_reaches_the_program() {
    let session = start("report-size");
    wait_for(&session, "ready");
    session.resize(GridSize {
        columns: 100,
        lines: 30,
        ..SIZE
    });
    wait_for(&session, "size:30 100");
    assert_eq!(session.snapshot().rows.len(), 30);
    assert_eq!(session.snapshot().columns, 100);
}

// PLAN §1.2 test 4: paste is bracketed only once the program asks for it.
#[test]
fn bracketed_paste_follows_what_the_program_asked_for() {
    let quiet = start("greet");
    wait_for(&quiet, "ready");
    assert!(!quiet.modes().bracketed_paste);

    let session = start("show-input");
    wait_for(&session, "ready");
    assert!(session.modes().bracketed_paste);
}

// PLAN §1.2 test 4, audit F1 #6: a pasted text cannot end the paste itself
// (ESC[201~ or the one-byte CSI), nor interrupt the program with ctrl+c.
#[test]
fn a_hostile_paste_stays_inside_its_brackets() {
    let session = start("raw-input");
    wait_for(&session, "ready");
    session.paste("a\x1b[201~b\u{9b}201~c\x03d");
    session.write(b"\r".to_vec());
    wait_for(&session, "^[[200~a[201~b201~cd^[[201~");
    assert!(!session.has_exited());
}

// PLAN §1.2 test 4: a program that did not ask for bracketed paste gets the
// text as if typed.
#[test]
fn a_paste_is_plain_text_for_a_program_that_did_not_ask_for_brackets() {
    let session = start("echo-lines");
    wait_for(&session, "ready");
    session.paste("uno\n");
    wait_for(&session, "got:uno");
}

fn key(key: &str) -> KeyPress<'_> {
    KeyPress {
        key,
        modifiers: KeyModifiers::default(),
    }
}

// Audit F1: DECCKM end to end. The arrows change when the program asks for
// application cursor keys.
#[test]
fn arrows_follow_the_cursor_key_mode_the_program_set() {
    let normal = start("raw-input");
    wait_for(&normal, "ready");
    assert!(normal.press(key("up")));
    normal.write(b"\r".to_vec());
    wait_for(&normal, "^[[A");

    let application = start("app-cursor");
    wait_for(&application, "ready");
    assert!(application.press(key("up")));
    application.write(b"\r".to_vec());
    wait_for(&application, "^[OA");
}

// PLAN §1.2 test 2: a text key is not sent as a key; the input method owns it.
#[test]
fn a_text_key_is_not_sent_by_the_key_path() {
    let session = start("raw-input");
    wait_for(&session, "ready");
    assert!(!session.press(key("ñ")));
}

// Audit F1: focus reports reach only the program that asked for them.
#[test]
fn focus_changes_are_reported_to_a_program_that_asked() {
    let session = start("focus-reports");
    wait_for(&session, "ready");
    session.report_focus(true);
    session.report_focus(false);
    session.write(b"\r".to_vec());
    wait_for(&session, "^[[I^[[O");

    let quiet = start("raw-input");
    wait_for(&quiet, "ready");
    quiet.report_focus(true);
    quiet.write(b"x\r".to_vec());
    let screen = wait_for(&quiet, "x");
    assert!(!screen.contains("^[[I"), "{screen}");
}

// Audit F1 #9: a color query is answered with the color the program set.
#[test]
fn a_color_query_returns_the_color_the_program_set() {
    let session = start("color-query");
    wait_for(&session, "ready");
    session.write(b"\r".to_vec());
    wait_for(&session, "]11;rgb:1111/2222/3333");
}

// Audit F1 #5: one column makes the emulator's reflow run away; the session
// never goes below two.
#[test]
fn a_session_is_never_narrower_than_two_columns() {
    let session = start("greet");
    wait_for(&session, "ready");
    session.resize(GridSize {
        columns: 1,
        lines: 0,
        ..SIZE
    });
    assert_eq!(session.size().columns, 2);
    assert_eq!(session.size().lines, 1);
    assert_eq!(session.snapshot().columns, 2);
}

// PLAN §1.2 test 4: selecting with the mouse, then copying.
#[test]
fn a_selection_yields_the_text_under_it() {
    let session = start("greet");
    wait_for(&session, "ready");
    let at = |column, right_half| GridPoint {
        line: 0,
        column,
        right_half,
    };
    session.start_selection(SelectionKind::Cells, at(0, false));
    session.extend_selection(at(2, true));
    assert_eq!(session.selected_text().as_deref(), Some("rea"));

    session.start_selection(SelectionKind::Words, at(1, false));
    assert_eq!(session.selected_text().as_deref(), Some("ready"));

    session.clear_selection();
    assert_eq!(session.selected_text(), None);
}

// PLAN §1.1: scroll
#[test]
fn scrolling_back_shows_older_output() {
    let session = start("echo-lines");
    wait_for(&session, "ready");
    for number in 0..40 {
        session.write(format!("line{number}\r").into_bytes());
    }
    let bottom = wait_for(&session, "got:line39");
    assert!(!bottom.contains("ready"));

    session.scroll(1000);
    wait_for(&session, "ready");
    session.scroll_to_bottom();
    wait_for(&session, "got:line39");
}

// PLAN §1.2 test 6: each pane has its own PTY.
#[test]
fn two_sessions_do_not_share_a_terminal() {
    let left = start("echo-lines");
    let right = start("echo-lines");
    wait_for(&left, "ready");
    wait_for(&right, "ready");
    left.write(b"izquierda\r".to_vec());
    let left_screen = wait_for(&left, "got:izquierda");
    assert!(left_screen.contains("got:izquierda"));
    let right_screen = crate::support::text(&right.snapshot());
    assert!(!right_screen.contains("izquierda"), "{right_screen}");
}

// PLAN §1.1: the session notices when its process ends.
#[test]
fn the_session_knows_when_the_program_ended() {
    let session = start("greet");
    wait_for(&session, "ready");
    assert!(!session.has_exited());
    // End of input on a terminal.
    session.write(vec![0x04]);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while !session.has_exited() {
        assert!(
            std::time::Instant::now() < deadline,
            "the program never ended"
        );
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}
