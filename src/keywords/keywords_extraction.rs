use rake::*;
pub fn get_keywords(lang:&str,text:&str)->Vec<KeywordScore>{
    let sw = get_stopwords_list(lang);
    let extractor = Rake::new(sw);
    let keywords = extractor.run(text);
    keywords
}
fn get_stopwords_list(lang:&str)->StopWords{
    let stop_words_list_path = "./db/stopwords-".to_owned()+lang+".txt";
    let sw = StopWords::from_file(&stop_words_list_path).unwrap();
    sw
}