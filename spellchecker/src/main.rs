#[doc="
Find possible corrections for misspelled words
It consists of two phases: 
1 Training module: consumes a corpus of correctly spelled words and counts the 
 number of occurrences of each word. 
2 Uses the results of the first to check individual words

INPUT:

The corpus file format is a sequence of text, including some punctuation
marks, written in ASCII :
    
    hello world,
    bye world


 The input format is a sequence of words — one per line

    hello
    hell
    word
    wordl
    wor
    wo

The input and corpus terminates with either EOF.

OUTPUT

For each word from standard in, prints one line. The line consists of just the 
word if it is spelled correctly. Otherwise, prints the word and the best 
improvement or “-” if there aren’t any improvements found.

  hello
  hell, hello
  word
  wordl, world
  wor, world
  wo, word
  w, -


Assumptions:

- Following operations are regarded as 1 edit:
    the deletion of one letter;
    the transposition of two neighboring letters;
    the replacement of one letter with another letter; and
    the insertion of a letter at any position.

- “Small edits” are those within 2 edits

- Fewer edits has higher priority


"]
use std::io::{BufRead,BufReader,Read, stdin};
use std::io::{Write, stdout};
use std::env;
use std::fs::File;

fn main() {
    let arg: Vec<_> = env::args().collect(); 
    if arg.len() != 2 {
        panic!("Argument Error!");
    } else {
        let f = File::open(arg[1].to_owned()).unwrap();
        let trie =  read_n_train_model(f);

        let words = read_words(stdin());
        write_correct_words(words, &trie, &mut stdout());

    }
}

type SubTries = std::collections::HashMap<char, Trie>;


// Use Trie to store the corpus and the frequency of words
struct Trie {
    count: usize,   // frequency of the word ending in this node
    children: SubTries,     // hashmap of subnodes
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


struct CheckResult {
    word:   Option<String>,
    count:  usize,
    edit:   usize,
}


fn check_spelling(trie: &Trie, word: String) ->Option<String> {
    let mut check = CheckResult {
        word: None,
        count: 0,
        edit: 2
    };

    let mut path = "".to_string();
    let to_go = word.chars().collect();
    search_with_k_edit(trie, &to_go, &mut path, 0, &mut check);

    return check.word;
}


fn search_with_k_edit(node: &Trie, to_go: &Vec<char>, mut path: &mut String, k: usize, mut check: &mut CheckResult) {
    if to_go.is_empty() {
        if k < check.edit && node.count > 0 || node.count > check.count && k == check.edit {
            // This word exists in corpus and is with higher frequency
            check.count = node.count;
            check.word = Some(path.to_owned());
            check.edit = k;
        } else {
            // Insert a letter at the env
            for (ch, sub) in &node.children {
                path.push(*ch);
                search_with_k_edit(&sub, &to_go, &mut path, k+1, &mut check);
                path.pop();
            }
        }
    } else {

        if let Some(sub) = node.children.get(&to_go[0]) {
        
            // Get match, no need to edit
            path.push(to_go[0].to_owned());
            search_with_k_edit(&sub, &to_go[1..].to_vec(), &mut path,  k, &mut check);
            path.pop();
        }
        
        // Need to edit

        if k >= check.edit {
            // Already found match word with fewer edits, no need to search 
            // ones edited more
            return;
        }

        // Delete a letter
        search_with_k_edit(&node, &to_go[1..].to_vec(), &mut path, k+1,&mut check);


        // Transpos adjacent letters
        if to_go.len() > 1 && node.children.contains_key(&to_go[1]) {
            let sub = node.children.get(&to_go[1]).unwrap();
            let mut go = to_go[2..].to_vec();
            go.insert(0, to_go[0].to_owned());

            path.push(to_go[1].to_owned());
            search_with_k_edit(&sub, &go, &mut path, k+1, &mut check);
            path.pop();
        }

        for (ch, sub) in &node.children {

            // Insert a letter
            path.push(*ch);
            search_with_k_edit(sub, &to_go, &mut path, k+1, &mut check);
            path.pop();


            // Replace a letter
            path.push(*ch);
            search_with_k_edit(sub, &to_go[1..].to_vec(), &mut path, k+1, &mut check);
            path.pop();


        }
            
    }
}



#[cfg(test)]
mod check_spelling_tests {
    use super::{check_spelling, insert_trie};
    use super::{Trie};

    #[test]
    fn find_or_not() {
        let mut t= Trie::new();

        assert_eq!(None,check_spelling(&t, "banana".to_string()));
        assert_eq!(None,check_spelling(&t, "apple".to_string()));

        insert_trie(&mut t, "apple".to_string());
        insert_trie(&mut t, "banana".to_string());

        assert_eq!(Some("banana".to_string()),check_spelling(&t, "banana".to_string()));
        assert_eq!(Some("apple".to_string()),check_spelling(&t, "apple".to_string()));
        assert_eq!(None,check_spelling(&t, "blueberry".to_string()));

        insert_trie(&mut t, "grapes".to_string());
        insert_trie(&mut t, "blueberry".to_string());

        assert_eq!(Some("blueberry".to_string()),check_spelling(&t, "blueberry".to_string()));
        assert_eq!(Some("grapes".to_string()),check_spelling(&t, "grapes".to_string()));
    }


    #[test]
    fn insert_spelling() {
        let mut t= Trie::new();

        insert_trie(&mut t, "an".to_string());
        insert_trie(&mut t, "apple".to_string());
        insert_trie(&mut t, "banana".to_string());
        insert_trie(&mut t, "ban".to_string());
        insert_trie(&mut t, "watermelon".to_string());


        assert_eq!(Some("an".to_string()),check_spelling(&t, "a".to_string()));
        assert_eq!(Some("an".to_string()),check_spelling(&t, "n".to_string()));
        assert_eq!(Some("ban".to_string()),check_spelling(&t, "b".to_string()));
        assert_eq!(Some("watermelon".to_string()),check_spelling(&t, "aterelon".to_string()));
        assert_eq!(Some("banana".to_string()),check_spelling(&t, "anana".to_string()));
        assert_eq!(Some("apple".to_string()),check_spelling(&t, "aple".to_string()));
    }


    #[test]
    fn replace_spelling() {
        let mut t= Trie::new();

        insert_trie(&mut t, "an".to_string());
        insert_trie(&mut t, "apple".to_string());
        insert_trie(&mut t, "banana".to_string());
        insert_trie(&mut t, "ban".to_string());
        insert_trie(&mut t, "watermelon".to_string());


        assert_eq!(Some("an".to_string()),check_spelling(&t, "en".to_string()));
        assert_eq!(Some("apple".to_string()),check_spelling(&t, "abpla".to_string()));
        assert_eq!(Some("ban".to_string()),check_spelling(&t, "ben".to_string()));
        assert_eq!(Some("watermelon".to_string()),check_spelling(&t, "watermalen".to_string()));
    }


    #[test]
    fn delete_spelling() {
        let mut t= Trie::new();

        insert_trie(&mut t, "an".to_string());
        insert_trie(&mut t, "apple".to_string());
        insert_trie(&mut t, "banana".to_string());
        insert_trie(&mut t, "ban".to_string());
        insert_trie(&mut t, "watermelon".to_string());


        assert_eq!(Some("an".to_string()),check_spelling(&t, "and".to_string()));
        assert_eq!(Some("apple".to_string()),check_spelling(&t, "iaiple".to_string()));
        assert_eq!(Some("ban".to_string()),check_spelling(&t, "abano".to_string()));
        assert_eq!(Some("watermelon".to_string()),check_spelling(&t, "watormaelon".to_string()));
    }


    #[test]
    fn transpose_spelling() {
        let mut t= Trie::new();

        insert_trie(&mut t, "an".to_string());
        insert_trie(&mut t, "apple".to_string());
        insert_trie(&mut t, "banana".to_string());
        insert_trie(&mut t, "ban".to_string());
        insert_trie(&mut t, "watermelon".to_string());


        assert_eq!(Some("an".to_string()),check_spelling(&t, "na".to_string()));
        assert_eq!(Some("apple".to_string()),check_spelling(&t, "appel".to_string()));
        assert_eq!(Some("ban".to_string()),check_spelling(&t, "bnae".to_string()));
        assert_eq!(Some("watermelon".to_string()),check_spelling(&t, "watermoeln".to_string()));
    }


    #[test]
    fn correct_base_on_frequency() {
        let mut t= Trie::new();

        insert_trie(&mut t, "an".to_string());
        insert_trie(&mut t, "apple".to_string());
        insert_trie(&mut t, "ape".to_string());
        insert_trie(&mut t, "banana".to_string());
        insert_trie(&mut t, "banny".to_string());

        assert_eq!(Some("ape".to_string()),check_spelling(&t, "aple".to_string()));
        assert_eq!(Some("banny".to_string()),check_spelling(&t, "banna".to_string()));

        insert_trie(&mut t, "apple".to_string());
        insert_trie(&mut t, "banana".to_string());

        assert_eq!(Some("apple".to_string()),check_spelling(&t, "aple".to_string()));
        assert_eq!(Some("banana".to_string()),check_spelling(&t, "banna".to_string()));
    }


}


fn read_n_train_model<R: Read>(reader: R) -> Trie {
    let mut trie = Trie::new();
    let mut lines = BufReader::new(reader).lines();
    let marks: &[_] = &[',','.','!','?',':',';','(',')','\'','\"','[',']','-'];

    while let Some(Ok(line)) = lines.next() {
        let words: Vec<&str> = line.split(' ').collect();

        for word in &words {
            let word = &(*word).trim_matches(marks).to_lowercase();
            if word.len() > 0 {
                insert_trie(&mut trie, (*word).to_owned());
            }
        }
    }

    return trie;
}

#[cfg(test)]
mod read_n_train_test {
    use super::{insert_trie, read_n_train_model, Trie};
    use std::io::{Read, Result};


    #[test]
    fn read_five_words() {
        let mock_read = StringReader::new("two three\n two three three\n".to_owned());
        let under_test = read_n_train_model(mock_read);
        let expected = number_trie();
        assert_eq_trie( &under_test, &expected);
    }


    #[test]
    fn read_words_uppercase() {
        let mock_read = StringReader::new("Two  tHree\n TWO THREE three\n".to_owned());
        let under_test = read_n_train_model(mock_read);
        let expected = number_trie();

        assert_eq_trie(&under_test, &expected);
    }


    #[test]
    fn read_words_n_marks() {
        let mock_read = StringReader::new("\'one\' two, : \"three\"\n two? three (three)\n".to_owned());
        let under_test = read_n_train_model(mock_read);
        let mut expected = number_trie();
        insert_trie(&mut expected, "one".to_owned());

        assert_eq_trie(&under_test, &expected);
    }


    fn number_trie() -> Trie {
        let mut t = Trie::new();
        insert_trie(&mut t,"two".to_owned());
        insert_trie(&mut t,"three".to_owned());
        insert_trie(&mut t,"three".to_owned());
        insert_trie(&mut t,"two".to_owned());
        insert_trie(&mut t,"three".to_owned());

        return t;

    }


    fn assert_eq_trie(a: &Trie, b: &Trie)
    {
        assert_eq!(a.count, b.count);
        for (ch, sub) in &a.children {
            if let Some(cor) = b.children.get(&ch){
                assert_eq_trie(&sub, &cor);               
            } else {
                assert!(false);
            }
        }
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


fn write_correct_words<W: Write>(words: Vec<String>, trie: &Trie, writer: &mut W) {

    let mut check_pairs: Vec<(String, Option<String>)> = Vec::new();
    for word in &words {
        check_pairs.push(( (*word).to_string(), check_spelling(&trie, (*word).to_string())));
    }

    for pair in &check_pairs {
        let correction = (pair.1).to_owned().unwrap_or("-".to_string());
        let word = (pair.0).to_owned();
        let line = match correction == word {
            true => word,
            false => word + ", " + &*correction,
        };

        if let Err(_) = (*writer).write(&*(format!("{}\n",line).into_bytes())){
            panic!("Fail writing");
        }
    }
}


#[cfg(test)]
mod write_correction_test {
    use super::{write_correct_words, insert_trie, Trie};

    #[test]
    fn write_empty_table() {
        let table = Trie::new();
        let mut buf: Vec<u8> = Vec::new();

        write_correct_words([].to_vec(), &table, &mut buf);
        assert_eq!(String::from_utf8(buf).unwrap(), "");
    }

    #[test]
    fn correct_two_string() {
        let table = number_trie();
        let mut buf: Vec<u8> = Vec::new();
        let words = vec!["thre".to_string(), "to".to_string()];
        write_correct_words(words, &table, &mut buf);
        assert_eq!(String::from_utf8(buf).unwrap(), "thre, three\nto, two\n");
    }

    #[test]
    fn correct_three_string() {
        let table = fruit_trie();
        let mut buf: Vec<u8> = Vec::new();
        let words = vec!["app".to_string(), "ban".to_string(), "watrmeoln".to_string()];

        write_correct_words(words, &table, &mut buf);
        assert_eq!(String::from_utf8(buf).unwrap(), "app, apple\nban, -\nwatrmeoln, watermelon\n");
    }


    fn number_trie() -> Trie {
        let mut t = Trie::new();
        insert_trie(&mut t,"two".to_owned());
        insert_trie(&mut t,"three".to_owned());
        insert_trie(&mut t,"three".to_owned());
        insert_trie(&mut t,"two".to_owned());
        insert_trie(&mut t,"three".to_owned());

        return t;

    }



    fn fruit_trie() -> Trie {
        let mut t = Trie::new();
        insert_trie(&mut t,"apple".to_owned());
        insert_trie(&mut t,"banana".to_owned());
        insert_trie(&mut t,"watermelon".to_owned());
        insert_trie(&mut t,"grapes".to_owned());
        insert_trie(&mut t,"apple".to_owned());

        return t;

    }

}