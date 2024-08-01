pub struct GeminiClient{
    api_key:String,
    model_name:String
}
impl GeminiClient{
    pub fn new(api_key:String,model_name:String)->Self{
        Self{api_key,model_name}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
