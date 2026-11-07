//! A form tied to a record of the program's own: each field reads its text from the record and writes it back as it is typed, with what is wrong in red words under its box.

use super::widget::{column, entry, form_label, label, Length, Node, Tone};
use crate::t;

type Get<T> = Box<dyn Fn(&T) -> String>;
type Set<T> = Box<dyn Fn(&mut T, &str) -> Result<(), &'static str>>;

struct Field<T> {
    name: String,
    get: Get<T>,
    set: Set<T>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Typed {
    pub field: usize,
    pub text: String,
}

pub struct Form<T> {
    fields: Vec<Field<T>>,
    texts: Vec<String>,
    problems: Vec<Option<&'static str>>,
}

impl<T> Default for Form<T> {
    fn default() -> Self {
        Form { fields: Vec::new(), texts: Vec::new(), problems: Vec::new() }
    }
}

impl<T> Form<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn text(mut self, name: impl Into<String>, get: impl Fn(&T) -> String + 'static, set: impl Fn(&mut T, &str) -> Result<(), &'static str> + 'static) -> Self {
        self.fields.push(Field { name: name.into(), get: Box::new(get), set: Box::new(set) });
        self.texts.push(String::new());
        self.problems.push(None);
        self
    }

    pub fn number(self, name: impl Into<String>, get: impl Fn(&T) -> f64 + 'static, set: impl Fn(&mut T, f64) + 'static) -> Self {
        self.text(name, move |r| get(r).to_string(), move |r, s| {
            let n: f64 = s.trim().parse().ok().filter(|n: &f64| n.is_finite()).ok_or_else(|| t!("kit-form-number"))?;
            set(r, n);
            Ok(())
        })
    }

    pub fn load(&mut self, record: &T) {
        for (i, f) in self.fields.iter().enumerate() {
            self.texts[i] = (f.get)(record);
            self.problems[i] = None;
        }
    }

    pub fn update(&mut self, typed: Typed, record: &mut T) {
        let Some(f) = self.fields.get(typed.field) else { return };
        self.problems[typed.field] = (f.set)(record, &typed.text).err();
        self.texts[typed.field] = typed.text;
    }

    pub fn text_of(&self, field: usize) -> Option<&str> {
        self.texts.get(field).map(String::as_str)
    }

    pub fn is_right(&self) -> bool {
        self.problems.iter().all(Option::is_none)
    }

    pub fn view<M: 'static>(&self, wrap: impl Fn(Typed) -> M + Clone + 'static) -> Node<M> {
        let rows = self.fields.iter().enumerate().map(|(i, f)| {
            let wrap = wrap.clone();
            let under = match self.problems[i] {
                Some(words) => label(words).tone(Tone::Problem),
                None => label(""),
            };
            column(vec![
                form_label(f.name.clone()),
                entry(self.texts[i].clone(), f.name.clone(), move |text| wrap(Typed { field: i, text })).width(Length::Fill),
                under.room_for(1),
            ]).gap(crate::theme::gaps().small)
        }).collect();
        column(rows).gap(crate::theme::gaps().roomy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Default, PartialEq)]
    struct Person {
        name: String,
        age: f64,
    }

    fn form() -> Form<Person> {
        Form::new()
            .text("Name", |p: &Person| p.name.clone(), |p, s| if s.trim().is_empty() { Err("Write a name.") } else { p.name = s.into(); Ok(()) })
            .number("Age", |p| p.age, |p, n| p.age = n)
    }

    #[test]
    fn a_form_reads_the_record_and_writes_back_only_what_is_right() {
        let mut person = Person { name: "John Smith".into(), age: 40.0 };
        let mut f = form();
        f.load(&person);
        assert_eq!((f.text_of(0), f.text_of(1)), (Some("John Smith"), Some("40")));
        f.update(Typed { field: 1, text: "41".into() }, &mut person);
        f.update(Typed { field: 0, text: " ".into() }, &mut person);
        assert_eq!(person, Person { name: "John Smith".into(), age: 41.0 });
        assert!(!f.is_right());
        f.update(Typed { field: 1, text: "forty".into() }, &mut person);
        assert_eq!(person.age, 41.0);
        let said = f.view(|t| t).said();
        assert!(said.contains(&"Write a name.".to_string()) && said.contains(&"Write a number, such as 12 or 3.5.".to_string()), "{said:?}");
        f.load(&person);
        assert!(f.is_right());
        f.update(Typed { field: 9, text: "x".into() }, &mut person);
    }
}
