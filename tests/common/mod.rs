use forge::model::PolicyRequirement;

pub fn make_req(text: &str, source_line: usize) -> PolicyRequirement {
    PolicyRequirement {
        stable_id: String::new(),
        text: text.to_string(),
        source_line,
        atom_index: 0,
        parent_text: None,
    }
}
