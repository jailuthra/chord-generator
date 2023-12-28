use itertools::Itertools;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use serde::Serialize;
use std::{collections::BTreeMap, ops::Add};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

const MAX_FRETS: u8 = 9;

#[derive(
    Debug,
    Copy,
    Clone,
    FromPrimitive,
    ToPrimitive,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    EnumIter,
    PartialOrd,
    Ord,
)]
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

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, EnumIter, PartialOrd, Ord)]
enum Chord {
    Major,
    Minor,
    Augmented,
    Diminished,
    Seventh,
    MajorSeventh,
    MinorSeventh,
    Sus2,
    Sus4,
    MinorMajorSeventh,
    DiminishedSeventh,
    MajorNinth,
    MinorNinth,
    AddNinth,
    AddEleventh,
    MinorSixth,
    MajorSixth,
    AddSixthAddNinth,
}

impl Chord {
    fn notes(&self, root: Note) -> Vec<Note> {
        match self {
            Chord::Major => vec![root, root + 4, root + 7],
            Chord::Minor => vec![root, root + 3, root + 7],
            Chord::Sus2 => vec![root, root + 2, root + 7],
            Chord::Sus4 => vec![root, root + 5, root + 7],
            Chord::Augmented => vec![root, root + 4, root + 8],
            Chord::Diminished => vec![root, root + 3, root + 6],
            Chord::MinorSixth => vec![root, root + 3, root + 7, root + 9],
            Chord::MajorSixth => vec![root, root + 4, root + 7, root + 9],
            Chord::Seventh => vec![root, root + 4, root + 7, root + 10],
            Chord::MajorSeventh => vec![root, root + 4, root + 7, root + 11],
            Chord::MinorSeventh => vec![root, root + 3, root + 7, root + 10],
            Chord::MinorMajorSeventh => vec![root, root + 3, root + 7, root + 11],
            Chord::DiminishedSeventh => vec![root, root + 3, root + 6, root + 9],
            Chord::MajorNinth => vec![root, root + 4, root + 7, root + 11, root + 14],
            Chord::MinorNinth => vec![root, root + 3, root + 7, root + 10, root + 14],
            Chord::AddNinth => vec![root, root + 4, root + 7, root + 14],
            Chord::AddEleventh => vec![root, root + 4, root + 7, root + 17],
            Chord::AddSixthAddNinth => vec![root, root + 4, root + 7, root + 9, root + 14],
        }
    }
}

type Tuning = [Note; 6];
const DEFAULT_TUNING: Tuning = [Note::E, Note::A, Note::D, Note::G, Note::B, Note::E];

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

fn compactness(fingering: &Fingering) -> i8 {
    let played: Vec<i8> = fingering
        .into_iter()
        .filter_map(|&f| {
            let x: i8 = f.into();
            if x > 0 {
                Some(x)
            } else {
                None
            }
        })
        .collect();
    if played.len() == 0 {
        return std::i8::MAX;
    }
    played.iter().max().unwrap() - played.iter().min().unwrap()
}

// TODO: This is temporary, we need to instead assign actual fingers and have a cost function for
// distance, cramping, crossing etc
fn fingering_score(fingering: &Fingering) -> u32 {
    let mut sum: u32 = 0;
    // prefer compact chords
    sum += (5 - compactness(fingering)) as u32;
    for finger in fingering {
        match finger.0 {
            // Open strings are best, give em max points :)
            Some(0) => sum += 15,
            // Closed strings are okay but better to have them at the start of the neck
            Some(x) => sum += (10 - x) as u32,
            // Muting is better than playing
            None => sum += 10,
        }
    }
    sum
}

fn get_played_notes(t: Tuning, fingering: Fingering) -> [Option<Note>; 6] {
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
        let played_notes = get_played_notes(t, fingering);

        // Check if all notes in this particular fingering are part of chord triad
        let mut all_played_notes_valid = true;
        for n in played_notes {
            match n {
                None => continue,
                Some(note) => {
                    if chord.notes(root).into_iter().find(|&x| x == note).is_none() {
                        all_played_notes_valid = false;
                        break;
                    }
                }
            }
        }

        // Check if all notes of the chord are being held
        let mut all_chord_notes_are_held = true;
        for note in chord.notes(root) {
            if played_notes
                .into_iter()
                .find(|&held| held == Some(note))
                .is_none()
            {
                all_chord_notes_are_held = false;
            }
        }

        if all_played_notes_valid && all_chord_notes_are_held {
            inversions.push(fingering);
        }

        if !next_fingering(&mut fingering) {
            break;
        }
    }
    inversions
}

// Is the fingering compact (true) or spread out across > 4 frets (false)
fn is_compact(fingering: &Fingering) -> bool {
    compactness(fingering) < 4
}

// Are the played strings contiguious (true) or have random unplayed strings in between (false)
fn is_contiguous(fingering: &Fingering) -> bool {
    let mut zone = 0;
    // xx12xx is valid, where first xx are zone0, 12 are zone1, and last xx are zone3
    for f in fingering {
        if zone == 0 {
            if f.0 != None {
                zone = 1;
            }
            continue;
        }
        if zone == 1 {
            if f.0 == None {
                zone = 2;
            }
            continue;
        }
        return false;
    }
    true
}

// Make sure at least four strings are being played, three note chords sound too empty
fn at_least_four_strings(fingering: &Fingering) -> bool {
    fingering.into_iter().filter(|f| f.0.is_some()).count() >= 4
}

fn main() {
    let mut m: BTreeMap<Note, BTreeMap<Chord, Vec<Fingering>>> = BTreeMap::new();

    for root in Note::iter() {
        m.insert(root, BTreeMap::new());
        for chord in Chord::iter() {
            let inversions: Vec<Fingering> = gen_inversions(root, chord, DEFAULT_TUNING)
                .into_iter()
                .filter(is_compact) // only compact
                .filter(is_contiguous) // only contiguous
                .filter(at_least_four_strings) // at least four played strings
                .sorted_by(|a, b| {
                    // sort the fingerings by descending score
                    u32::cmp(&fingering_score(b), &fingering_score(a))
                })
                .collect();
            // insert list of inversions for this particular chord
            m.get_mut(&root).unwrap().insert(chord, inversions.clone());
        }
    }
    println!("{}", serde_json::to_string_pretty(&m).unwrap());
}
