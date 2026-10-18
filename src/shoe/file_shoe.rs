use crate::card::Card;
use crate::shoe::CardShoe;
use std::path::Path;
use std::{fmt, fs, io};

pub struct FileShoe {
    cards: Vec<Card>,
    curr_pos: usize,
    stride: usize,
    ended: bool,
}

impl FileShoe {
    pub fn new(file_path: &Path) -> Result<FileShoe, io::Error> {
        let cards = fs::read(file_path)?
            .iter()
            .filter_map(|&n| match n {
                b'1'..=b'9' => Some(Card(n - b'0')),
                b'T' => Some(Card(10)),
                b'A' => Some(Card(1)),
                b' ' | b'\n' | b'\t' => None,
                _ => panic!("Invalid byte {n:#02x}"),
            })
            .collect();

        Ok(FileShoe {
            cards,
            curr_pos: 0,
            stride: 1,
            ended: false,
        })
    }
}

impl Iterator for FileShoe {
    type Item = Card;

    fn next(&mut self) -> Option<Self::Item> {
        let pos = self.curr_pos;

        if self.ended {
            return None;
        }

        self.curr_pos += self.stride;
        if self.curr_pos >= self.cards.len() {
            self.stride += 1;
            if self.stride >= self.cards.len() / 10 {
                self.ended = true;
            }
            self.curr_pos -= self.cards.len();
        }

        Some(self.cards[pos])
    }
}

impl CardShoe for FileShoe {
    fn try_pick(&mut self) -> Option<Card> {
        Some(
            self.next()
                .expect("Reached maximum number of cards for this FileShoe"),
        )
    }

    fn try_pick_first(&mut self, card: Card) -> Option<Card> {
        Some(
            self.find(|&c| c == card)
                .expect("Reached maximum number of cards for this FileShoe"),
        )
    }

    fn reshuffle(&mut self) {
        unimplemented!("FileShoe cannot be reshuffled")
    }

    fn force_true_count(&mut self, _true_count: f32) {
        unimplemented!("Cannot force true count to a FileShoe")
    }

    fn adjust_running_count(&mut self, _rel_rc: i32) {
        unimplemented!("Cannot adjust running count on a FileShoe")
    }
}

impl fmt::Display for FileShoe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;

        for (i, card) in self.cards.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", card)?;
        }

        write!(f, "]")
    }
}

#[cfg(test)]
mod tests {
    use crate::card::Card;
    use crate::shoe::CardShoe;
    use crate::shoe::file_shoe::FileShoe;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn it_loads_a_file_shoe_text() {
        let path = std::env::temp_dir().join(format!(
            "freebj-test-cards-{}-{}.txt", std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        let mut file = File::create_new(&path).expect("Couldn't create test file");
        file.write_all(b"16\t8 TA4\n")
            .expect("Couldn't write test file");
        drop(file);

        let mut shoe = FileShoe::new(&path).expect("Couldn't create FileShoe");
        std::fs::remove_file(&path).expect("Couldn't remove test file");

        assert_eq!(shoe.pick(), Card(1));
        assert_eq!(shoe.pick(), Card(6));
        assert_eq!(shoe.pick(), Card(8));
        assert_eq!(shoe.pick(), Card(10));
        assert_eq!(shoe.pick(), Card(1));
        assert_eq!(shoe.pick(), Card(4));
    }
}
