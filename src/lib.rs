#![warn(rust_2018_idioms)]

use std::cell::Cell;
use std::result;
use std::vec::Vec;

#[derive(Debug)]
pub struct Arc<'a> {
    weight: u32,
    place: &'a Place,
}

impl<'a> Arc<'a> {
    pub fn new(place: &'a Place, weight: u32) -> result::Result<Self, &str> {
        if weight < 1 {
            Err("weight must be greater 0")
        } else {
            Ok(Self { weight, place })
        }
    }

    pub fn can_provide_required_tokens(&self) -> bool {
        self.place.tokens.get() >= self.weight
    }

    pub fn consume_tokens(&self) {
        self.place.tokens.set(self.place.tokens.get() - self.weight);
    }

    pub fn produce_tokens(&self) {
        self.place.tokens.set(self.place.tokens.get() + self.weight);
    }
}

#[derive(Debug)]
pub struct Place {
    tokens: Cell<u32>,
    name: String,
}

impl Place {
    pub fn new(tokens: u32, name: &str) -> Self {
        Self {
            tokens: Cell::new(tokens),
            name: name.to_string(),
        }
    }

    pub fn tokens(&self) -> u32 {
        self.tokens.get()
    }
}

#[derive(Default)]
pub struct Transition<'a> {
    input_arcs: Vec<&'a Arc<'a>>,
    output_arcs: Vec<&'a Arc<'a>>,
    expression: Option<Box<dyn Fn() -> bool + 'a>>,
}

impl<'a> Transition<'a> {
    pub fn new() -> Self {
        Self {
            input_arcs: Vec::new(),
            output_arcs: Vec::new(),
            expression: None,
        }
    }

    pub fn new_with_expression(func: impl Fn() -> bool + 'a) -> Self {
        Self {
            input_arcs: Vec::new(),
            output_arcs: Vec::new(),
            expression: Some(Box::new(func)),
        }
    }

    pub fn add_input(&mut self, arc: &'a Arc<'a>) {
        self.input_arcs.push(arc);
    }

    pub fn add_output(&mut self, arc: &'a Arc<'a>) {
        self.output_arcs.push(arc);
    }

    pub fn is_enabled(&self) -> bool {
        let all_arcs_enabled = self
            .input_arcs
            .iter()
            .all(|arc| arc.can_provide_required_tokens());

        if let Some(ref f) = self.expression {
            all_arcs_enabled && f()
        } else {
            all_arcs_enabled
        }
    }

    pub fn fire(&mut self) {
        for v in self.input_arcs.iter_mut() {
            v.consume_tokens();
        }

        for v in self.output_arcs.iter_mut() {
            v.produce_tokens();
        }
    }
}

#[derive(Default)]
pub struct Petrinet<'a> {
    transitions: Vec<Transition<'a>>,
}

impl<'a> Petrinet<'a> {
    pub fn new() -> Self {
        Self {
            transitions: Vec::new(),
        }
    }

    pub fn add_transition(&mut self, transition: Transition<'a>) {
        self.transitions.push(transition)
    }

    pub fn step(&mut self) {
        for transition in &mut self.transitions {
            if transition.is_enabled() {
                transition.fire();
                break;
            }
        }
    }
}

pub struct RandomTransitionScheduler {}

impl RandomTransitionScheduler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn schedule(&mut self, transitions: &[Transition<'_>]) {
        todo!("implement schedule");
    }
}

impl Default for RandomTransitionScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::seq::SliceRandom;

    #[test]
    fn it_works() {
        let place = super::Place::new(1, "a");
        let arc = super::Arc::new(&place, 1).expect("weight greater than zero");

        let mut transition = super::Transition::new();
        transition.add_input(&arc);

        transition.fire();
    }

    #[test]
    fn test_transition_without_expression() {
        let p1 = super::Place::new(2, "p1");
        let p2 = super::Place::new(2, "p2");
        let p3 = super::Place::new(0, "p3");

        let a1 = super::Arc::new(&p1, 2).expect("weight greater than zero");
        let a2 = super::Arc::new(&p2, 1).expect("weight greater than zero");
        let a3 = super::Arc::new(&p3, 2).expect("weight greater than zero");

        let mut t = super::Transition::new();
        t.add_input(&a1);
        t.add_input(&a2);

        t.add_output(&a3);

        assert!(t.is_enabled());

        t.fire();

        assert_eq!(0, p1.tokens());
        assert_eq!(1, p2.tokens());
        assert_eq!(2, p3.tokens(), "number of tokens on place p3");

        assert!(
            !t.is_enabled(),
            "must not fire because input tokens are not sufficient anymore"
        );

        assert_eq!(0, p1.tokens());
        assert_eq!(1, p2.tokens());
        assert_eq!(2, p3.tokens(), "number of tokens on place p3");
    }

    #[test]
    fn test_transition_with_expression() {
        let p1 = Place::new(2, "p1");
        let p2 = Place::new(2, "p2");
        let p3 = Place::new(0, "p3");

        let a1 = Arc::new(&p1, 2).expect("weight greater than zero");
        let a2 = Arc::new(&p2, 1).expect("weight greater than zero");
        let a3 = Arc::new(&p3, 2).expect("weight greater than zero");

        // with expression which evaluates to true
        let mut t = Transition::new_with_expression(|| true);
        t.add_input(&a1);
        t.add_input(&a2);

        t.add_output(&a3);

        assert!(
            t.is_enabled(),
            "must be enabled since transition expression is true"
        );

        t.fire();

        assert_eq!(2, p3.tokens(), "number of tokens on place p3");

        let p1 = Place::new(2, "p1");
        let p2 = Place::new(2, "p2");
        let p3 = Place::new(0, "p3");

        let a1 = Arc::new(&p1, 2).expect("weight greater than zero");
        let a2 = Arc::new(&p2, 1).expect("weight greater than zero");
        let a3 = Arc::new(&p3, 2).expect("weight greater than zero");

        // with expression which evaluates to false
        let mut t = Transition::new_with_expression(|| false);
        t.add_input(&a1);
        t.add_input(&a2);

        t.add_output(&a3);

        assert!(
            !t.is_enabled(),
            "must not be enabled since transition expression is false"
        );
    }

    #[test]
    fn test_arc() {
        let place = super::Place::new(2, "p");
        let arc = super::Arc::new(&place, 2).expect("weight greater than zero");
        assert!(arc.can_provide_required_tokens());

        let place = super::Place::new(1, "p");
        let arc = super::Arc::new(&place, 2).expect("weight greater than zero");
        assert!(!arc.can_provide_required_tokens());

        let place = super::Place::new(3, "p");
        let arc = super::Arc::new(&place, 2).expect("weight greater than zero");
        assert!(arc.can_provide_required_tokens());

        let place = super::Place::new(2, "");
        assert!(super::Arc::new(&place, 0).is_err());
    }

    use rand::distributions::Distribution;

    #[test]
    fn test_rng() {
        let between = rand::distributions::Uniform::new(0, 10);
        let mut rng = rand::thread_rng();

        let x = between.sample(&mut rng);

        assert!(x >= 0 && x < 10);

        let mut v = vec![1, 2, 3, 4];
        v.shuffle(&mut rng);

        let mut count = 0;
        for _v in v.choose_multiple(&mut rng, v.len()) {
            count += 1;
        }
        assert_eq!(4, count);
    }

    #[test]
    fn test_petrinet() {
        let p1 = super::Place::new(2, "P1");
        let p2 = super::Place::new(2, "P2");

        let p3 = super::Place::new(0, "P3");

        let a1 = super::Arc::new(&p1, 2).expect("weight greater than zero");
        let a2 = super::Arc::new(&p2, 1).expect("weight greater than zero");

        let a3 = super::Arc::new(&p3, 2).expect("weight greater than zero");

        let mut t = super::Transition::new();
        t.add_input(&a1);
        t.add_input(&a2);

        t.add_output(&a3);

        let mut net = super::Petrinet::new();

        net.add_transition(t);

        net.step();

        net.step();
    }

    // https://en.wikipedia.org/wiki/Petri_net#/media/File:Detailed_petri_net.png
    #[test]
    fn test_more_complex_petrinet() {
        let p1 = super::Place::new(1, "P1");
        let p2 = super::Place::new(0, "P2");
        let p3 = super::Place::new(2, "P3");
        let p4 = super::Place::new(1, "P4");

        let a1 = super::Arc::new(&p1, 1).expect("weight greater than zero");

        let a2 = super::Arc::new(&p2, 1).expect("weight greater than zero");
        let a3 = super::Arc::new(&p3, 1).expect("weight greater than zero");

        let a4 = super::Arc::new(&p2, 1).expect("weight greater than zero");
        let a5 = super::Arc::new(&p3, 1).expect("weight greater than zero");

        let a6 = super::Arc::new(&p4, 1).expect("weight greater than zero");

        let a7 = super::Arc::new(&p1, 1).expect("weight greater than zero");

        let mut t1 = super::Transition::new();
        let mut t2 = super::Transition::new();

        t1.add_input(&a1);
        t1.add_output(&a2);
        t1.add_output(&a3);

        t2.add_input(&a4);
        t2.add_input(&a5);
        t2.add_output(&a6);
        t2.add_output(&a7);

        let mut petri = super::Petrinet::new();
        petri.add_transition(t1);
        petri.add_transition(t2);

        petri.step();

        assert_eq!(0, p1.tokens());
        assert_eq!(1, p2.tokens());
        assert_eq!(3, p3.tokens());

        petri.step();

        assert_eq!(1, p1.tokens());
        assert_eq!(0, p2.tokens());
        assert_eq!(2, p3.tokens());
        assert_eq!(2, p4.tokens());

        petri.step();
        petri.step();

        assert_eq!(3, p4.tokens());
    }
}
