use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Trie {
    children: HashMap<char, Trie>,
    end_of_word: bool
}

impl Trie {
    pub fn new () -> Self {
        Self {
            children: HashMap::new(),
            end_of_word: false
        }
    }

    pub fn insert (&mut self, word: &str) {
        let mut curr = self;

        for ch in word.chars() {
            curr = curr.children.entry(ch).or_insert_with(Self::new);
        }

        curr.end_of_word = true;
    }

    pub fn with_prefix (&self, word: &str) -> Vec<String> {
        let mut curr = self;
        let mut res = Vec::new();
        if word.is_empty() { return res };

        for ch in word.chars() {
            match curr.children.get(&ch) {
                Some(next) => curr = next,
                None => return res,
            }
        };

        Self::collect_words(curr, word, &mut res);
        res

    }

    pub fn collect_words (node: &Self, prefix: &str, res: &mut Vec<String>) {
        if node.end_of_word {
            res.push(prefix.to_string().clone());
        }

        for (ch, node) in &node.children {
            let mut pref = prefix.to_string().clone();
            pref.push(*ch);
            Self::collect_words(node, &pref, res);
        }
    }
}

#[cfg(test)]
mod trie_tests {
    use super::*;

    #[test]
    #[should_panic]
    fn trie_insert_test () {
        let mut tr = Trie::new();

        tr.insert("Hi mom");
        tr.insert("Hi mother");
        tr.insert("Hi father");
        tr.insert("apple");

        assert_eq!(tr.with_prefix("Hi mo"), vec![
            "Hi mom".to_string(),
            "Hi mother".to_string()
        ]);
    }
}
