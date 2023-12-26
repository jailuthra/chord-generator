use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use serde::Serialize;
use std::{collections::HashMap, ops::Add};

const MAX_FRETS: u8 = 9;

#[derive(Debug, Copy, Clone, FromPrimitive, ToPrimitive, PartialEq, Eq, Hash, Serialize)]
enum Note {
    C = 0,
    CSharp,
    D,
    DSharp,
    E,
    F,
    FSharp,
    G,
    GSharp,
    A,
    ASharp,
    B,
}

impl Add<u8> for Note {
    type Output = Self;

    fn add(self, rhs: u8) -> Self::Output {
        FromPrimitive::from_u8((ToPrimitive::to_u8(&self).unwrap() + rhs) % 12).unwrap()
    }
}

impl Add<Finger> for Note {
    type Output = Option<Self>;

    fn add(self, rhs: Finger) -> Self::Output {
        match rhs.0 {
            None => None,
            Some(val) => Some(self + val),
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize)]
enum Chord {
    Major,
    Minor,
}

impl Chord {
    fn notes(&self, root: Note) -> Vec<Note> {
        match self {
            Chord::Major => vec![root, root + 4, root + 7],
            Chord::Minor => vec![root, root + 3, root + 7],
        }
    }
}

type Tuning = [Note; 6];

#[derive(Copy, Clone, Debug)]
struct Finger(Option<u8>);

impl Into<char> for Finger {
    fn into(self) -> char {
        match self.0 {
            None => 'x',
            Some(v) => char::from_digit(v as u32, 10).unwrap(),
        }
    }
}

impl Into<i8> for Finger {
    fn into(self) -> i8 {
        match self.0 {
            None => -1,
            Some(v) => v as i8,
        }
    }
}

impl Serialize for Finger {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_i8(<Finger as Into<i8>>::into(*self))
    }
}

type Fingering = [Finger; 6];

fn next_fingering(fingering: &mut Fingering) -> bool {
    for f in fingering.iter_mut().rev() {
        match f.0 {
            None => {
                *f = Finger(Some(0));
                return true;
            }
            Some(MAX_FRETS) => {
                *f = Finger(None);
            }
            Some(x) => {
                *f = Finger(Some(x + 1));
                return true;
            }
        }
    }
    // if we haven't returned by this point we have gone beyond the maximum possible fingerings, so
    // return false to user
    false
}

fn get_held_notes(t: Tuning, fingering: Fingering) -> [Option<Note>; 6] {
    let mut notes = [None; 6];
    for (i, f) in fingering.into_iter().enumerate() {
        notes[i] = t[i] + f;
    }
    notes
}

fn gen_inversions(root: Note, chord: Chord, t: Tuning) -> Vec<Fingering> {
    let mut inversions = Vec::new();
    let mut fingering: Fingering = [Finger(None); 6];

    loop {
        let held_notes = get_held_notes(t, fingering);

        // Check if all notes in this particular fingering are part of chord triad
        let mut all_held_notes_valid = true;
        for n in held_notes {
            match n {
                None => continue,
                Some(note) => {
                    if chord.notes(root).into_iter().find(|&x| x == note).is_none() {
                        all_held_notes_valid = false;
                        break;
                    }
                }
            }
        }

        // Check if all notes of the chord are being held
        let mut all_chord_notes_are_held = true;
        for note in chord.notes(root) {
            if held_notes
                .into_iter()
                .find(|&held| held == Some(note))
                .is_none()
            {
                all_chord_notes_are_held = false;
            }
        }

        if all_held_notes_valid && all_chord_notes_are_held {
            inversions.push(fingering);
        }

        if !next_fingering(&mut fingering) {
            break;
        }
    }
    inversions
}

fn main() {
    let t: Tuning = [Note::E, Note::A, Note::D, Note::G, Note::B, Note::E];
    let mut m: HashMap<Note, HashMap<Chord, Vec<Fingering>>> = HashMap::new();
    for root in [Note::D, Note::C] {
        m.insert(root, HashMap::new());
        for chord in [Chord::Major, Chord::Minor] {
            let inversions = gen_inversions(root, chord, t);
            m.get_mut(&root).unwrap().insert(chord, inversions.clone());
        }
    }
    println!("{}", serde_json::to_string(&m).unwrap());
}
