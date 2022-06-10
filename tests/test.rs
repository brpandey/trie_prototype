#[cfg(test)]
mod tests {

    use trie_prototype::trie::Trie;


    #[test]
    fn monolith() {
        let mut t = Trie::new();

        {
            t.insert("anthem", Some(1));
            t.insert("anthems", Some(9));
            t.insert("anti", Some(2));
            t.insert("anthemion", Some(7));
            t.insert("anthemis", Some(77));
        }

        println!("1 trie is {:#?}", t);

        let v1 = t.search("anthem");
        let v2 = t.search("ant");

        println!("search anthem, result {:?}", v1);
        println!("search ant, result {:?}", v2);

        //    let result = t.insert("anthem", 98);

        //    println!("old result {:?}", result);

        let v1 = t.search("anthem");
        let v2 = t.search("anthemion");

        println!("search anthem, result {:?}", v1);
        println!("search anthemion, result {:?}", v2);

        let results = t.all_keys("ant");

        println!("\nall keys result {:?}", &results);


        let result = t.longest_prefix("anthemion");
        println!("\nlongest prefix result {:?}", &result);

        let removed;

        {
            //       removed = t.remove("anthemion");
        }

        println!("2 trie is {:#?}", t);

        //    println!("2 removed is {:?}", removed);


        {
            removed = t.remove("anthem");
        }

        println!("3 trie is {:#?}", t);

        println!("3 removed is {:?}", removed);

        t.keys();

        println!("4 trie is {:#?}", t);

    }
}
