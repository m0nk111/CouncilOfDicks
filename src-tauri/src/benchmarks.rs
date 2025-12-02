use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Benchmark {
    pub id: String,
    pub category: String,
    pub question: String,
    pub trap_explanation: String,
    pub difficulty: String,
}

pub fn get_all_benchmarks() -> Vec<Benchmark> {
    vec![
        Benchmark {
            id: "logic_sisters".to_string(),
            category: "Logic Trap".to_string(),
            question: "Sally has 3 brothers. Each brother has 2 sisters. How many sisters does Sally have?".to_string(),
            trap_explanation: "Models often multiply 3 * 2 = 6. Correct answer is 1 (there are 2 girls total, Sally is one).".to_string(),
            difficulty: "Medium".to_string(),
        },
        Benchmark {
            id: "physics_feathers".to_string(),
            category: "Common Misconception".to_string(),
            question: "Which is heavier: a kilogram of lead or a kilogram of feathers?".to_string(),
            trap_explanation: "Models often explain density instead of weight, or fail to acknowledge they are equal weight.".to_string(),
            difficulty: "Easy".to_string(),
        },
        Benchmark {
            id: "tokenization_strawberry".to_string(),
            category: "Tokenization Blind Spot".to_string(),
            question: "How many times does the letter 'r' appear in the word 'strawberry'?".to_string(),
            trap_explanation: "Tokenizers often split 'strawberry' into 'straw' and 'berry', obscuring the letters. Models often say 2. Correct is 3.".to_string(),
            difficulty: "Hard".to_string(),
        },
        Benchmark {
            id: "math_apples".to_string(),
            category: "Sequential Logic".to_string(),
            question: "I have 3 apples. I eat 2. I buy 5 more. I give 1 to my friend. How many apples do I have now?".to_string(),
            trap_explanation: "Simple arithmetic (3-2+5-1=5), but models can lose track of the state.".to_string(),
            difficulty: "Easy".to_string(),
        },
        Benchmark {
            id: "hallucination_president".to_string(),
            category: "Hallucination Trigger".to_string(),
            question: "Who was the President of the United States in 1650?".to_string(),
            trap_explanation: "The US didn't exist in 1650. Models might hallucinate a name or refer to a colonial governor.".to_string(),
            difficulty: "Medium".to_string(),
        },
        Benchmark {
            id: "logic_killers".to_string(),
            category: "Inverse Logic".to_string(),
            question: "If it takes 5 machines 5 minutes to make 5 widgets, how long would it take 100 machines to make 100 widgets?".to_string(),
            trap_explanation: "Intuitive answer is 100 minutes. Correct answer is 5 minutes (each machine takes 5 mins per widget).".to_string(),
            difficulty: "Medium".to_string(),
        },
        Benchmark {
            id: "monty_hall".to_string(),
            category: "Probability".to_string(),
            question: "I am on a game show. There are 3 doors. Behind one is a car, behind the others, goats. I pick Door 1. The host opens Door 3, which has a goat. He asks if I want to switch to Door 2. Should I switch?".to_string(),
            trap_explanation: "Counter-intuitive probability. Switching gives 2/3 chance of winning. Staying gives 1/3.".to_string(),
            difficulty: "Hard".to_string(),
        }
    ]
}

#[tauri::command]
pub fn get_benchmarks() -> Vec<Benchmark> {
    get_all_benchmarks()
}
