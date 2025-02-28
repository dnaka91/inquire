use super::Password;
use crate::{
    terminal::crossterm::CrosstermTerminal,
    ui::{Backend, RenderConfig},
    validator::{ErrorMessage, Validation},
};
use crossterm::event::{KeyCode, KeyEvent};

macro_rules! text_to_events {
    ($text:expr) => {{
        $text.chars().map(KeyCode::Char)
    }};
}

macro_rules! password_test {
    ($(#[$meta:meta])? $name:ident,$input:expr,$output:expr,$prompt:expr) => {
        #[test]
        $(#[$meta])?
        fn $name() {
            let read: Vec<KeyEvent> = $input.into_iter().map(KeyEvent::from).collect();
            let mut read = read.iter();

            let mut write: Vec<u8> = Vec::new();
            let terminal = CrosstermTerminal::new_with_io(&mut write, &mut read);
            let mut backend = Backend::new(terminal, RenderConfig::default()).unwrap();

            let ans = $prompt.prompt_with_backend(&mut backend).unwrap();

            assert_eq!($output, ans);
        }
    };
}

password_test!(
    empty,
    vec![KeyCode::Enter],
    "",
    Password::new("").without_confirmation()
);

password_test!(
    single_letter,
    vec![KeyCode::Char('b'), KeyCode::Enter],
    "b",
    Password::new("").without_confirmation()
);

password_test!(
    letters_and_enter,
    text_to_events!("normal input\n"),
    "normal input",
    Password::new("").without_confirmation()
);

password_test!(
    letters_and_enter_with_emoji,
    text_to_events!("with emoji 🧘🏻‍♂️, 🌍, 🍞, 🚗, 📞\n"),
    "with emoji 🧘🏻‍♂️, 🌍, 🍞, 🚗, 📞",
    Password::new("").without_confirmation()
);

password_test!(
    input_and_correction,
    {
        let mut events = vec![];
        events.append(&mut text_to_events!("anor").collect());
        events.push(KeyCode::Backspace);
        events.push(KeyCode::Backspace);
        events.push(KeyCode::Backspace);
        events.push(KeyCode::Backspace);
        events.append(&mut text_to_events!("normal input").collect());
        events.push(KeyCode::Enter);
        events
    },
    "normal input",
    Password::new("").without_confirmation()
);

password_test!(
    input_and_excessive_correction,
    {
        let mut events = vec![];
        events.append(&mut text_to_events!("anor").collect());
        events.push(KeyCode::Backspace);
        events.push(KeyCode::Backspace);
        events.push(KeyCode::Backspace);
        events.push(KeyCode::Backspace);
        events.push(KeyCode::Backspace);
        events.push(KeyCode::Backspace);
        events.push(KeyCode::Backspace);
        events.push(KeyCode::Backspace);
        events.push(KeyCode::Backspace);
        events.push(KeyCode::Backspace);
        events.append(&mut text_to_events!("normal input").collect());
        events.push(KeyCode::Enter);
        events
    },
    "normal input",
    Password::new("").without_confirmation()
);

password_test!(
    input_correction_after_validation,
    {
        let mut events = vec![];
        events.append(&mut text_to_events!("1234567890").collect());
        events.push(KeyCode::Enter);
        events.push(KeyCode::Backspace);
        events.push(KeyCode::Backspace);
        events.push(KeyCode::Backspace);
        events.push(KeyCode::Backspace);
        events.push(KeyCode::Backspace);
        events.append(&mut text_to_events!("yes").collect());
        events.push(KeyCode::Enter);
        events
    },
    "12345yes",
    Password::new("")
        .without_confirmation()
        .with_validator(|ans: &str| match ans.len() {
            len if len > 5 && len < 10 => Ok(Validation::Valid),
            _ => Ok(Validation::Invalid(ErrorMessage::Default)),
        })
);

password_test!(
    input_confirmation_same,
    {
        let mut events = vec![];
        events.append(&mut text_to_events!("1234567890").collect());
        events.push(KeyCode::Enter);
        events.append(&mut text_to_events!("1234567890").collect());
        events.push(KeyCode::Enter);
        events
    },
    "1234567890",
    Password::new("")
);

password_test!(
    #[should_panic(expected = "Custom stream of characters has ended")]
    input_confirmation_different,
    {
        let mut events = vec![];
        events.append(&mut text_to_events!("1234567890").collect());
        events.push(KeyCode::Enter);
        events.append(&mut text_to_events!("abcdefghij").collect());
        events.push(KeyCode::Enter);
        events
    },
    "",
    Password::new("")
);
