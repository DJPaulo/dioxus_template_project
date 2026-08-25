use serde::{Serialize, Deserialize};

// -------------------------------------------------------------------
// Environment Variable Names
// -------------------------------------------------------------------
pub const ENV_VARIABLE_NAME_CLIENT_URL: &str = "RUST_CLIENT_URL";
pub const ENV_VARIABLE_NAME_SERVER_URL: &str = "RUST_SERVER_URL";
pub const ENV_VARIABLE_NAME_DB_PATH: &str = "RUST_DB_PATH";
pub const ENV_VARIABLE_NAME_INITIAL_ADMIN_EMAIL: &str = "RUST_ADMIN_EMAIL";
pub const ENV_VARIABLE_NAME_INITIAL_ADMIN_PASSWORD: &str = "RUST_ADMIN_PASSWORD";

// -------------------------------------------------------------------
// User login and api error handling structures
// -------------------------------------------------------------------
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum Role {
    Admin,
    User,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub role: Role,
    pub is_active: bool,
    pub change_password: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChangePasswordRequest {
    pub user: User,
    pub old_password: String,
    pub new_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PasswordResetRequest {
    pub token: String,
    pub user: User,
    pub new_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenCheckRequest {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub message: String,
}


// -------------------------------------------------------------------
// Questionnaire structures
// -------------------------------------------------------------------
pub type QuestionId = u32;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnswerType {
    Text,
    Integer,
    Decimal,
    Boolean,
    Date,
    SingleSelect { options: Vec<String> },
    MultipleSelect { options: Vec<String> },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnswerValue {
    Text(String),
    Integer(i64),
    Decimal(f64),
    Boolean(bool),
    Date(time::Date),
    SingleSelect(usize),
    MultipleSelect(Vec<usize>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Condition {
    Equals(AnswerValue),
    NotEquals(AnswerValue),
    GreaterThan(f64),
    LessThan(f64),
    IsTrue,
    IsFalse,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Branch {
    pub condition: Condition,
    pub next_question: QuestionId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Question {
    pub id: QuestionId,
    pub text: String,
    pub question_order: u32,
    pub answer_type: AnswerType,
    pub branches: Vec<Branch>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Questionnaire {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub questions: Vec<Question>,
    pub start_question: QuestionId,
}



