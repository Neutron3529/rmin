use rmin::*;
use std::collections::HashMap;
struct Table {
    headers:Vec<Vec<String,String>>,
    template:std::collections::HashMap<String,Vec<String>>,
    data:Vec<(String,Vec<f64>)>
}
impl Table {
    pub fn new(header:Vec<String>)->Self {
        Self {
            headers:vec![header.map(|x|(x,"{}")).collect()],
            ..Default::default()
        }
    }
    pub fn template(&mut self, name: String, template:Vec<String>) {
        self.template[name]=template;
    }
    pub fn template(&mut self, name: String, template:Vec<String>) {
        self.template[name]=template;
    }
}
#[export]
pub fn main(){
    
}