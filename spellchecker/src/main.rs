#[doc="
Find possible corrections for misspelled words.

Assumptions:

"]
use std::io::{BufRead,BufReader,Read};
use std::io::{Write, stdout};
use std::env;
use std::fs::File;

fn main() {
    let arg: Vec<_> = env::args().collect(); 
    if arg.len() != 2 {
        panic!("Argument Error!");
    } else {
        let f = File::open(arg[1].to_owned()).unwrap();
        let htable =  read_n_count_words(f);
        write_word_frequency(htable, &mut stdout());

    }
}

type SubTries = std::collections::HashMap<char, Trie>;

struct Trie {
    count: usize,
    //children: std::collections::HashMap<char, Trie>,
    children: SubTries,
}

impl Trie{
    fn new() -> Self{
        Trie {
            count: 0,
            children: SubTries::new(),
        }

    }

    fn insert(&mut self, path: Vec<char>) {
        if path.is_empty() {
            self.count += 1;
        } else {
            self.children.entry(path[0].to_owned()).or_insert(Trie::new()).insert(path[1..].to_vec());
        }
    }

    fn search(&self, path: Vec<char>) -> bool {
        if path.is_empty() {
            match self.count {
                0 => return false,
                _ => return true,
            }
        } else {
            if let Some(child) = self.children.get(&path[0]){
                return child.search(path[1..].to_vec());
            } else {
                return false;
            }

        }
    }
}


fn insert_trie(t: &mut Trie, word: String){
    t.insert(word.chars().collect());
}

fn search_trie(t: &Trie, word: String) -> bool {
    return t.search(word.chars().collect());
}

#[cfg(test)]
mod tries_tests {
    use super::{insert_trie, search_trie};
    use super::{Trie};

    #[test]
    fn trie_insert() {
        let mut t = Trie::new();
        insert_trie(&mut t, "a".to_string());
        assert_eq!(1, t.children.get(&'a').unwrap().count);
        assert_eq!(1, t.children.len());

        insert_trie(&mut t, "an".to_string());
        insert_trie(&mut t, "an".to_string());
        if let Some(c) = t.children.get(&'a') {
            assert_eq!(2, c.children.get(&'n').unwrap().count);
            assert_eq!(1, c.children.len());
        } else {
            assert!(false);
        }

    }

    #[test]
    fn trie_search() {
        let mut t = Trie::new();
        insert_trie(&mut t, "a".to_string());
        insert_trie(&mut t, "an".to_string());
        insert_trie(&mut t, "apple".to_string());

        assert!(search_trie(&t, "a".to_string()));
        assert!(search_trie(&t, "an".to_string()));
        assert!(search_trie(&t, "apple".to_string()));
        assert!(!search_trie(&t, "app".to_string()));

    }
}

type CountTable = std::collections::HashMap<String, usize>;

#[allow(dead_code)]
fn increment_word(map: &mut CountTable, word: String) {
    *map.entry(word).or_insert(0) += 1;
}

#[cfg(test)]
mod increment_word_tests {
    use super::{increment_word, CountTable};

    #[test]
    fn inserts_if_empty() {
        let mut h = CountTable::new();
        increment_word(&mut h, "one".to_owned());

        assert_eq!(Some(&1), h.get("one"));
        assert_eq!(1, h.len());
    }

    #[test]
    fn increments_if_present() {
        let mut under_test = fixture();
        let mut expected   = fixture();

        increment_word(&mut under_test, "three".to_owned());
        expected.insert("three".to_owned(), 4);

        assert_eq!(expected, under_test);
    }

    #[test]
    fn insert_if_absent() {
        let mut under_test = fixture();
        let mut expected   = fixture();

        increment_word(&mut under_test, "one".to_owned());
        expected.insert("one".to_owned(), 1);

        assert_eq!(expected, under_test);
    }

    fn fixture() -> CountTable {
        let mut h = CountTable::new();
        h.insert("two".to_owned(), 2);
        h.insert("three".to_owned(), 3);

        return h;

    }
}

fn read_n_count_words<R: Read>(reader: R) -> CountTable {
    let mut table = CountTable::new();
    let mut lines = BufReader::new(reader).lines();
    let marks: &[_] = &[',','.','!','?',':',';','(',')','\'','\"','[',']','-'];

    while let Some(Ok(line)) = lines.next() {
        let words: Vec<&str> = line.split(' ').collect();

        for word in &words {
            let word = &(*word).trim_matches(marks).to_lowercase();
            if word.len() > 0 {
                increment_word(&mut table, (*word).to_owned());
            }
        }
    }

    return table;
}

#[cfg(test)]
mod read_n_count_test {
    use super::{read_n_count_words, CountTable};
    use std::io::{Read, Result};


    #[test]
    fn read_five_words() {
        let mock_read = StringReader::new("two three\n two three three\n".to_owned());
        let under_test = read_n_count_words(mock_read);
        let expected = fixture();

        assert_eq!(under_test.to_owned(), expected);
    }


    #[test]
    fn read_words_uppercase() {
        let mock_read = StringReader::new("Two  tHree\n TWO THREE three\n".to_owned());
        let under_test = read_n_count_words(mock_read);
        let expected = fixture();

        assert_eq!(under_test.to_owned(), expected);
    }


    #[test]
    fn read_words_n_marks() {
        let mock_read = StringReader::new("\'one\' two, : \"three\"\n two? three (three)\n".to_owned());
        let under_test = read_n_count_words(mock_read);
        let mut expected = fixture();
        expected.insert("one".to_owned(), 1);

        assert_eq!(under_test.to_owned(), expected);
    }


    fn fixture() -> CountTable {
        let mut h = CountTable::new();
        h.insert("two".to_owned(), 2);
        h.insert("three".to_owned(), 3);

        return h;

    }

    struct StringReader {
        contents: Vec<u8>,
        position: usize,
    }

    impl StringReader {
        fn new(s: String) -> Self {
            StringReader {
                contents: s.into_bytes(),
                position: 0,
            }
        }
    }

    impl Read for StringReader {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
            let mut count = 0;

            while self.position < self.contents.len() && count < buf.len() {
                buf[count] = self.contents[self.position];
                count += 1;
                self.position += 1;
            }

            return Ok(count);
        }
    }
}


fn read_words<R: Read>(reader: R) -> Vec<String> {
    let mut words: Vec<String> = Vec::new();
    let mut lines = BufReader::new(reader).lines();

    while let Some(Ok(line)) = lines.next() {
        if !line.is_empty() {
            words.push(String::from(line.trim()));
        }

    }

    return words;
} 


#[cfg(test)]
mod read_words_test {
    use super::{read_words};
    use std::io::{Read, Result};

    #[test]
    fn read_three_words() {
        assert_read(vec!["One".to_owned(), "Two".to_owned(), "Three".to_owned()], "One\nTwo\nThree\n");
    }

    #[test]
    fn read_empty() {
        assert_read(vec![], "\n");
    }


    fn assert_read(expected: Vec<String>, input: &str) {
        let mock_read = StringReader::new(input.to_owned());
        let read_in = read_words(mock_read);
        assert_eq!(expected, read_in);
    }

    struct StringReader {
        contents: Vec<u8>,
        position: usize,
    }

    impl StringReader {
        fn new(s: String) -> Self {
            StringReader {
                contents: s.into_bytes(),
                position: 0,
            }
        }
    }

    impl Read for StringReader {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
            let mut count = 0;

            while self.position < self.contents.len() && count < buf.len() {
                buf[count] = self.contents[self.position];
                count += 1;
                self.position += 1;
            }

            return Ok(count);
        }
    }
}


fn write_word_frequency<W: Write>(table: CountTable, writer: &mut W) {

    let mut wf_pairs: Vec<(String, usize)> = Vec::new();
    for (word, freq) in &table {
        wf_pairs.push((word.to_owned(), freq.to_owned()));
    }

    wf_pairs.sort_by(|a, b| b.1.cmp(&(a.1)));

    for wf in &wf_pairs {
        if let Err(_) = (*writer).write(&*(format!("{}\t:\t{}\n",wf.0, wf.1).into_bytes())){
            panic!("Fail writing");
        }
    }
}


#[cfg(test)]
mod write_counttable_test {
    use super::{write_word_frequency,CountTable};

    #[test]
    fn write_empty_table() {
        let table = CountTable::new();
        let mut buf: Vec<u8> = Vec::new();

        write_word_frequency(table, &mut buf);
        assert_eq!(String::from_utf8(buf).unwrap(), "");
    }

    #[test]
    fn write_two_string() {
        let table = fixture();
        let mut buf: Vec<u8> = Vec::new();

        write_word_frequency(table, &mut buf);
        assert_eq!(String::from_utf8(buf).unwrap(), "three\t:\t3\ntwo\t:\t2\n");
    }


    #[test]
    fn write_three_string() {
        let mut table = fixture();
        let mut buf: Vec<u8> = Vec::new();

        table.insert("one".to_owned(), 1);
        write_word_frequency(table, &mut buf);
        assert_eq!(String::from_utf8(buf).unwrap(), "three\t:\t3\ntwo\t:\t2\none\t:\t1\n");
    }


    fn fixture() -> CountTable {
        let mut h = CountTable::new();
        h.insert("two".to_owned(), 2);
        h.insert("three".to_owned(), 3);

        return h;

    }


}