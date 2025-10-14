use std::{path::PathBuf, str::FromStr};
use uuid::Uuid;

/// 题目类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestionType {
    /// 单选题
    SingleChoice,
    /// 阅读理解
    Reading,
    /// 完形填空
    ClozeTest,
}

impl QuestionType {
    /// 获取题目类型的字符串表示
    pub fn as_str(&self) -> &'static str {
        match self {
            QuestionType::SingleChoice => "单选题",
            QuestionType::Reading => "阅读理解",
            QuestionType::ClozeTest => "完形填空",
        }
    }
}
impl FromStr for QuestionType {
    type Err = ();

    fn from_str(input: &str) -> Result<QuestionType, Self::Err> {
        match input {
            "单选题" => Ok(QuestionType::SingleChoice),
            "阅读理解" => Ok(QuestionType::Reading),
            "完形填空" => Ok(QuestionType::ClozeTest),
            _ => Err(()),
        }
    }
}

/// 题目模板提示词
pub struct PromptTemplate {
    question_type: QuestionType,
}

impl PromptTemplate {
    /// 创建新的提示模板
    pub fn new(question_type: QuestionType) -> Self {
        Self { question_type }
    }

    /// 获取对应类型的提示词
    pub fn get_prompt(&self) -> String {
        match self.question_type {
            QuestionType::SingleChoice => Self::get_single_choice_prompt(),
            QuestionType::Reading => Self::get_reading_prompt(),
            QuestionType::ClozeTest => Self::get_cloze_test_prompt(),
        }
    }

    /// 单选题提示词
    fn get_single_choice_prompt() -> String {
        String::from(
            r#"请你把我给你的题目转换成如下格式的 JavaScript，格式如下：
var Questions = [
    {
        stem: `Which of the following is a <span class="underline fillblank" data-blank-id="593417796829762300" contenteditable="false" style="text-indent: 0; border-bottom: 1px solid #f6c908;display:inline-block;min-width: 40px;max-width: 80px;"><input type="text" style="display:none">   </span> language?`,
        "options": [
            "Python",
            "HTML", 
            "CSS",
            "HTTP"
        ],
        "answer": 0, // 答案索引：A
        analysis: "考点：编程语言识别。分析：Python是一种高级编程语言，广泛用于数据科学、人工智能等领域。故答案为：programming"
    }
];

注意事项：
1. 题目不要带题号
2. data-blank-id每次要不同
3. 答案选项不要带有A、B、C、D前缀
4. 解析要用中文，格式：考点，分析，故答案为
5. 不要带有```javascript ```，只输出代码就可以了。我不用代码块包裹
"#,
        )
    }

    /// 阅读理解提示词  
    fn get_reading_prompt() -> String {
        String::from(
            r#"输出模式如下：
// 模板，段落两端对齐，首行缩进，字体字号不变
// 在OCR时，注意把试卷中的不相关内容去掉，避免干扰
// 字体和字大小要和此模板一致，不要改变

var newContent = `
    <p style="text-align: justify; text-indent: 2em;">
        Lorem ipsum dolor sit amet, consectetur adipiscing elit. 
        <span class="number fillblank" contenteditable="false" data-blank-id="1" 
              style="display: inline-block;width:40px;height: 20px;line-height: 20px;border-bottom: 2px solid #000;text-align:center">
        </span> 
        Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
    </p>
    <p style="text-align: justify; text-indent: 2em;">
        Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.
        <span class="number fillblank" contenteditable="false" data-blank-id="2"
              style="display: inline-block;width:40px;height: 20px;line-height: 20px;border-bottom: 2px solid #000;text-align:center">
        </span>
        Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur.
    </p>
`;"#,
        )
    }

    /// 完形填空提示词
    fn get_cloze_test_prompt() -> String {
        String::from(
            r#"// 完形填空模板，段落两端对齐，首行缩进，字体字号不变
// 在OCR时，注意把试卷中的不相关内容去掉，避免干扰
// 字体和字大小要和此模板一致，不要改变

var newContent = `
    <h2 style="text-align: center;">Student Aspirations</h2>
    <p style="text-align: justify; text-indent: 2em;">
        "Who would you like to change your life with if you can?" Last week, we asked many middle school students this 
        <span class="number fillblank" contenteditable="false" data-blank-id="31" 
              style="text-indent:0; display: inline-block;width:40px;height: 20px;line-height: 20px;border-bottom: 2px solid #000;text-align:center">31</span>. 
        The following are some of their 
        <span class="number fillblank" contenteditable="false" data-blank-id="32" 
              style="text-indent:0; display: inline-block;width:40px;height: 20px;line-height: 20px;border-bottom: 2px solid #000;text-align:center">32</span>.
    </p>
    <p style="text-align: justify; text-indent: 2em;">
        <strong>Zhang Yike, 12</strong><br>
        I want to change my life with my friend Wang Xiaohui. She is 
        <span class="number fillblank" contenteditable="false" data-blank-id="33" 
              style="text-indent:0; display: inline-block;width:40px;height: 20px;line-height: 20px;border-bottom: 2px solid #000;text-align:center">33</span> 
        in England now.
    </p>
`;

// 完形填空题目数据，每个对象包含选项和答案索引
// 注意：'answer' 字段是从0开始的数字索引 (0=A, 1=B, 2=C, 3=D)
var Questions = [
    { 
        "options": ["reason", "question", "word", "way"], 
        "answer": 1, 
        "analysis": "考点：名词辨析。分析：根据上下文这是一个提问，所以这里应该填问题。故答案为question。" 
    },
    { 
        "options": ["answers", "problems", "questions", "changes"], 
        "answer": 0, 
        "analysis": "考点：名词辨析。分析：既然前面是问题，后面紧接着就是学生们的回答。故答案为answers。" 
    },
    { 
        "options": ["study", "studies", "to study", "studying"], 
        "answer": 3, 
        "analysis": "考点：现在分词作表语。分析：句意为她现在正在英国学习。be + doing 表示正在进行。故答案为studying。" 
    },
    { 
        "options": ["work", "walk", "wish", "wash"], 
        "answer": 2, 
        "analysis": "考点：动词辨析。分析：我也希望参观那些地方。wish to do sth. 意为希望做某事。故答案为wish。" 
    }
];"#,
        )
    }
}

/// 附加代码生成器
pub struct AdditionalCodeGenerator {
    question_type: QuestionType,
}

impl AdditionalCodeGenerator {
    /// 创建新的附加代码生成器
    pub fn new(question_type: QuestionType) -> Self {
        Self { question_type }
    }

    /// 获取附加代码
    pub fn get_code(&self) -> String {
        match self.question_type {
            QuestionType::SingleChoice => self.get_single_choice_code(),
            QuestionType::Reading => self.get_reading_code(),
            QuestionType::ClozeTest => self.get_cloze_test_code(),
        }
    }

    /// 单选题附加代码
    fn get_single_choice_code(&self) -> String {
        String::from(
            r#"// 单选题交互逻辑
function initSingleChoiceQuestions() {
    // 题目渲染逻辑
    console.log("单选题初始化完成");
}"#,
        )
    }

    /// 阅读理解附加代码  
    fn get_reading_code(&self) -> String {
        String::from(
            r#"// 阅读理解填空逻辑
function initReadingQuestions() {
    // 填空题交互逻辑
    console.log("阅读理解初始化完成");
}"#,
        )
    }

    /// 完形填空附加代码
    fn get_cloze_test_code(&self) -> String {
        String::from(
            r#"// 完形填空交互逻辑
function initClozeTestQuestions() {
    // 完形填空逻辑
    console.log("完形填空初始化完成");
}"#,
        )
    }
}

/// 题目结构体
#[derive(Debug, Clone)]
pub struct Question {
    /// 题目类型
    pub question_type: QuestionType,
    /// 唯一标识符
    pub id: Uuid,
    /// 提示词
    pub prompt: String,
    /// 题目内容
    pub stem: String,
    /// 图片路径（可选）
    pub img_path: Option<PathBuf>,
    /// 输出结果（可选）
    pub output: Option<String>,
    /// 附加代码（可选）
    pub additional_code: String,
}
 #[allow(dead_code)]
impl Question {
    pub fn set_stem(&mut self, stem: String) {
        self.stem = stem;
    }
    pub fn set_output_example_for_test(&mut self) {
        self.output = Some(String::from("This is an example output."));
    }
    pub fn set_model_reply(&mut self, reply: String) {
        self.output = Some(reply);
    }
    pub fn set_img_path(&mut self, path: Option<PathBuf>) {
        self.img_path = path;
    }

    /// 创建新的题目
    pub fn new(question_type: QuestionType, stem: String, img_path: Option<PathBuf>) -> Self {
        let prompt_template = PromptTemplate::new(question_type);
        let prompt = prompt_template.get_prompt();

        Self {
            question_type,
            id: Uuid::new_v4(),
            prompt,
            stem,
            img_path,
            output: None,
            additional_code: AdditionalCodeGenerator::new(question_type).get_code(),
        }
    }
    pub fn prompt_stem(&self) -> String {
        self.stem.clone() + &self.prompt.clone()
    }
    /// 获取题目ID
    pub fn get_id(&self) -> &Uuid {
        &self.id
    }

    /// 获取题目类型
    pub fn get_type(&self) -> QuestionType {
        self.question_type
    }

    /// 获取提示词
    pub fn get_prompt(&self) -> &str {
        &self.prompt
    }

    /// 获取题目内容
    pub fn get_stem(&self) -> &str {
        &self.stem
    }

    /// 获取图片路径
    pub fn get_img_path(&self) -> Option<&PathBuf> {
        self.img_path.as_ref()
    }

    /// 获取输出结果
    pub fn get_output(&self) -> Option<&str> {
        self.output.as_deref()
    }
    pub fn get_final_output(&self) -> String {
        let mut final_output = String::new();
        if let Some(ref output) = self.output {
            final_output.push_str(output);
        }
        if !self.additional_code.is_empty() {
            final_output.push_str("\n\n");
            final_output.push_str(&self.additional_code);
        }
        final_output
    }
    /// 检查题目是否完整
    pub fn is_complete(&self) -> bool {
        !self.stem.is_empty() && self.output.is_some()
    }

    /// 获取题目摘要信息
    pub fn get_summary(&self) -> String {
        let status = if self.is_complete() {
            "已完成"
        } else {
            "未完成"
        };
        let img_info = if self.img_path.is_some() {
            "有图片"
        } else {
            "无图片"
        };

        format!(
            "[{}] {} - {} - {} - ID: {}",
            self.question_type.as_str(),
            status,
            img_info,
            &self.stem.chars().take(50).collect::<String>(),
            self.id
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_question_type_as_str() {
        assert_eq!(QuestionType::SingleChoice.as_str(), "单选题");
        assert_eq!(QuestionType::Reading.as_str(), "阅读理解");
        assert_eq!(QuestionType::ClozeTest.as_str(), "完形填空");
    }

    #[test]
    fn test_question_creation() {
        let question = Question::new(
            QuestionType::SingleChoice,
            "这是一个测试题目".to_string(),
            None,
        );

        assert_eq!(question.get_type(), QuestionType::SingleChoice);
        assert_eq!(question.get_stem(), "这是一个测试题目");
        assert!(!question.is_complete()); // 没有输出结果，所以不完整
    }

    #[test]
    fn test_prompt_template() {
        let template = PromptTemplate::new(QuestionType::SingleChoice);
        let prompt = template.get_prompt();

        assert!(prompt.contains("JavaScript"));
        assert!(prompt.contains("Questions"));
    }

    #[test]
    fn test_additional_code_generator() {
        let generator = AdditionalCodeGenerator::new(QuestionType::ClozeTest);
        let code = generator.get_code();

        assert!(code.contains("完形填空"));
        assert!(code.contains("function"));
    }
}
