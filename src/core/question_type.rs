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
    /// 单项听力理解
    ListeningSingle,
    /// 听力复合题
    ListeningCompound,
    ///　多个填空
    MutiTiankong,
}

impl QuestionType {
    /// 获取题目类型的字符串表示
    pub fn as_str(&self) -> &'static str {
        match self {
            QuestionType::SingleChoice => "单选题",
            QuestionType::Reading => "阅读理解",
            QuestionType::ClozeTest => "完形填空",
            QuestionType::ListeningSingle => "单项听力理解",
            QuestionType::ListeningCompound => "听力复合题",
            QuestionType::MutiTiankong => "多个填空题",
        }
    }
}
impl FromStr for QuestionType {
    type Err = ();

    fn from_str(input: &str) -> Result<QuestionType, Self::Err> {
        match input {
            "单选题" => Ok(QuestionType::SingleChoice),
            "阅读理解" => Ok(QuestionType::Reading),
            "完型填空" => Ok(QuestionType::ClozeTest),
            "单项听力理解" => Ok(QuestionType::ListeningSingle),
            "听力复合题" => Ok(QuestionType::ListeningCompound),
            "多个填空题" => Ok(QuestionType::MutiTiankong),
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
            QuestionType::ListeningSingle => Self::get_listening_single_prompt(),
            QuestionType::ListeningCompound => Self::get_listening_compound_prompt(),
            QuestionType::MutiTiankong => Self::get_muti_tiankong_prompt(),
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
//正文中 中英文之间请保持空格。如grammars (语法) and
//请直接输出如下格式的JavaScript代码，不要回复其他内容。不要带有```javascript ```，只输出代码就可以了。我不用代码块包裹
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
`;

// !!! 注意：'answer' 字段是 从0开始的数字索引 (0=A, 1=B, 2=C) !!!
// 通用示例题目数据，适用于各类阅读理解或单选题
var Questions = [
    {
        "stem": "",//这里不要带题号
        "options": [
            "Python",//答案中不要带有A.
            "HTML",
            "CSS",
            "HTTP"
        ],
        "answer": 0, // 答案索引：A
        "analysis": "Python is a programming language. HTML and CSS are markup and style sheet languages, while HTTP is a protocol."//解析要用中文。格式要分为：考点，分析，故答案为：
    },
    {
        "stem": "What does 'AI' stand for?",
        "options": [
            "Artificial Intelligence",//原文保持一致~如果原文中每个选项有.那就在选项后面加英文句号，没有就算了。总之和原文保持一致

            "Automated Input",
            "Advanced Internet",
            "Analog Interface"
        ],
        "answer": 0, // 答案索引：A
        "analysis": "AI stands for Artificial Intelligence, which refers to the simulation of human intelligence by machines."
    }
];
"#,
        )
    }

    /// 完形填空提示词
    fn get_cloze_test_prompt() -> String {
        String::from(
            r#"
//正文中 中英文之间请保持空格。如grammars (语法) and
//请直接输出如下格式的JavaScript代码，不要回复其他内容。不要带有```javascript ```，不要带有```javascript ```。只输出代码就可以了。我不用代码块包裹
// 完形填空模板，段落两端对齐，首行缩进，字体字号不变
// 在OCR时，注意把试卷中的不相关内容去掉，避免干扰
// 字体和字大小要和此模板一致，不要改变
//do not 带有```javascript ```
var newContent = `
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
        "analysis": "考点：名词辨析。分析：根据上下文这是一个提问，所以这里应该填问题。故选B。" 
    },
    { 
        "options": ["answers", "problems", "questions", "changes"], 
        "answer": 0, 
        "analysis": "考点：名词辨析。分析：既然前面是问题，后面紧接着就是学生们的回答。故选A。" 
    },
];"#,
        )
    }

    fn get_listening_single_prompt() -> String {
        String::from(
            r#"
            "请你把我给你的题目转换成如下格式的 JavaScript，格式如下：
            //请直接输出如下格式的JavaScript代码，不要回复其他内容。不要带有```javascript ```，只输出代码就可以了。我不用代码块包裹
var Questions = [
    {
        "stem": "When did the dialogue most probably take place?",
        "options": [
            "In winter.",//答案中不要带有A.
            "In autumn.",
            "In spring."
        ],
        "answer": 1,
        "analysis": "考点：听力季节推断。原文：W: This is my favorite time of the year. Many leaves turn yellow. They are so beautiful. M: I prefer spring because it’s a season of new life and growth. 分析：对话中女士说“This is my favorite time of the year. Many leaves turn yellow.”（这是一年中我最喜欢的时候。很多叶子变黄了。），树叶变黄是秋天的典型特征。故答案为：B。"//解析要用中文。格式要分为：原文，分析，故答案为：（每个题目 的解析要有原文）。格式要用html的语法来写。
    },
    {
        "stem": "Why do some teenagers feel stressed?",
        "options": [
            "They have too many exams.",
            "They have too much homework to do.",
            "They don't know how to make friends with others."
        ],
        "answer": 0,
        "analysis": "考点：听力原因理解。原文：W: I hear some teenagers often feel stressed. M: Yes. They are too busy with their exams. 分析：对话中男士解释青少年感到压力的原因时说“They are too busy with their exams.”（他们忙于应付考试。），这与选项A“他们有太多的考试”意思相符。故答案为：A。"
    }
]
"#,
        )
    }

    fn get_listening_compound_prompt() -> String {
        String::from(
            r#"
        //请直接输出如下格式的JavaScript代码，不要回复其他内容。不要带有```javascript ```，只输出代码就可以了。我不用代码块包裹
var newContent = `
111
`;

var Questions = [
      {
        "stem": "What does the boy advise the girl to do?",
        "options": [
            "To take more exercise.",
            "To have a good rest.",
            "To stay at home."
        ],
        "answer": 0,
        "analysis": "考点：听力细节理解。原文：M: Take it easy. I think you can take more exercise. Keeping healthy is necessary. 分析：当女士表达了对爬山的担忧后，男士建议说“I think you can take more exercise”（我认为你可以多做些锻炼）。故答案为：A。"//解析要用中文。格式要分为：原文，分析，故答案为：（每个题目 的解析要有原文）。格式要用html的语法来写。
    }

          {
        "stem": "What does the boy advise the girl to do?",
        "options": [
            "To take more exercise.",
            "To have a good rest.",
            "To stay at home."
        ],
        "answer": 0,
        "analysis": "考点：听力细节理解。原文：M: Take it easy. I think you can take more exercise. Keeping healthy is necessary. 分析：当女士表达了对爬山的担忧后，男士建议说“I think you can take more exercise”（我认为你可以多做些锻炼）。故答案为：A。"//解析要用中文。格式要分为：原文，分析，故答案为：（每个题目 的解析要有原文）。格式要用html的语法来写。
    }

]
"#,
        )
    }

    fn get_muti_tiankong_prompt() -> String {
        String::from(
            r#"
//请直接输出如下格式的JavaScript代码，不要回复其他内容。不要带有```javascript ```，只输出代码就可以了。我不用代码块包裹
var Questions = [
    {
        stem: `Which of the following is a <span class="underline fillblank" data-blank-id="593417796829762300" contenteditable="false" style="text-indent: 0; border-bottom: 1px solid #f6c908;display:inline-block;min-width: 40px;max-width: 80px;"><input type="text" style="display:none">   </span> language?`, //这里不要带题号.这里的data-blank-id每次不要相同
        题型类型: "语音题",
        answer: ["programming"],
        analysis: "考点：编程语言识别。分析：Python是一种高级编程语言，广泛用于数据科学、人工智能等领域。故答案为：programming", //解析要用中文。格式要分为：考点，分析，故答案为：
    },
    {
        stem: `The capital of France is <span class="underline fillblank" data-blank-id="593417796829762301" contenteditable="false" style="text-indent: 0; border-bottom: 1px solid #f6c908;display:inline-block;min-width: 40px;max-width: 80px;"><input type="text" style="display:none">   </span>.`,
        题型类型: "填空题",
        answer: ["Paris"],
        analysis: "考点：世界地理常识。分析：巴黎是法国的首都和最大城市，也是法国的政治、经济、文化中心。故答案为：Paris"
    },
    {//如果检测到是一个文章。且一个题目里面有多个空的，用下面这种格式
            stem:`Good morning my name is (1) <span class="underline fillblank" data-blank-id="593417796829762302" contenteditable="false" style="text-indent: 0; border-bottom: 1px solid #f6c908;display:inline-block;min-width: 40px;max-width: 80px;"><input type="text" style="display:none">   </span> I am from (2) <span class="underline fillblank" data-blank-id="593417796829762303" contenteditable="false" style="text-indent: 0; border-bottom: 1px solid #f6c908;display:inline-block;min-width: 40px;max-width: 80px;"><input type="text" style="display:none">   </span>`,
            题型类型: "填空题",
            answer: ["John", "Canada"],
            analysis: "1. 考点：.....。分析：根据常见的自我介绍格式，名字是John. 故答案为：John,<br>2. 分析：.......。国家是Canada。故答案为： Canada"
    },
];
"#,
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
            QuestionType::ListeningSingle => self.get_listening_single_code(),
            QuestionType::ListeningCompound => self.get_listening_compound_code(),
            QuestionType::MutiTiankong => self.get_muti_tiankong_code(),
        }
    }

    /// 单选题附加代码
    fn get_single_choice_code(&self) -> String {
        String::from(
            r#" 
/**
 * 等待指定毫秒数
 * @param {number} ms - 等待的时间（毫秒）
 */
const delay = (ms) => new Promise(resolve => setTimeout(resolve, ms));

//MARK： 使用XPath查找包含指定文本的元素
function clickBlankFillingElement(type) {
    // XPath表达式：查找class包含"tag"且包含指定文本的元素
    var xpath = "//*[contains(@class,'tag') and contains(text(),'" + type + "')]";

    // 执行XPath查询
    var result = document.evaluate(
        xpath,
        document,
        null,
        XPathResult.FIRST_ORDERED_NODE_TYPE,
        null
    );

    // 如果找到元素，点击它
    if (result.singleNodeValue) {
        result.singleNodeValue.click();
        console.log(`成功点击 ${type} 标签元素`);
        return true;
    } else {
        console.log(`未找到包含 '${type}' 文本的标签元素`);
        return false;
    }
}
//
// 完整的操作流程 - 设置为单选题
async function operateElements() {
    console.log("开始设置题型为单选题...");

    // 1. 点击下拉框 - 查找当前选中的题型
    var selectDiv = document.querySelector('div[title]');
    if (!selectDiv) {
        // 备用选择器
        selectDiv = document.querySelector('.ant-select-selection-selected-value');
        if (!selectDiv) {
            selectDiv = document.querySelector('.ant-select-selection__rendered');
        }
    }

    if (selectDiv) {
        selectDiv.click();
        console.log("✅ 已点击题型下拉框");

        // 2. 等待下拉菜单出现，然后选择单选题
        await new Promise(resolve => {
            setTimeout(function () {
                var options = document.querySelectorAll('li.ant-select-dropdown-menu-item');
                for (var i = 0; i < options.length; i++) {
                    if (options[i].textContent.trim() === '单选题') {
                        options[i].click();
                        console.log("✅ 已选择单选题");
                        break;
                    }
                }
                resolve();
            }, 200);
        });
        // 2.5. 点击“选择题”标签
        await delay(200);
        const tagSpans = document.querySelectorAll('span.tag');
        for (let span of tagSpans) {
            if (span.textContent.trim() === '选择题') {
            span.click();
            console.log('✅ 已点击“选择题”标签');
            break;
            }
        }
        await delay(200);
        // 3. 等待一下确保选择生效
        await new Promise(resolve => setTimeout(resolve, 300));

        console.log("✅ 题型设置完成");
        return true;
    } else {
        console.error("❌ 未找到题型下拉框");
        return false;
    }
}
/**
 * 封装好的填充函数，用于向可编辑的 div 填入内容
 * @param {HTMLElement} container - 题目总容器
 * @param {string} placeholder - 通过 placeholder 文本来精确定位输入框
 * @param {string} text - 要填充的 HTML 内容
 */
async function fillEditableDiv(container, placeholder, text) {
    // 多种选择器策略
    let inputElement = null;
    
    // 策略1: 精确匹配 placeholder
    let selector = `[contenteditable="true"][placeholder="${placeholder}"]`;
    inputElement = container.querySelector(selector);
    
    if (!inputElement) {
        // 策略2: 查找包含 placeholder 文本的元素
        selector = `[contenteditable="true"]`;
        const editableElements = container.querySelectorAll(selector);
        for (let element of editableElements) {
            if (element.getAttribute('placeholder') && element.getAttribute('placeholder').includes(placeholder)) {
                inputElement = element;
                break;
            }
        }
    }
    
    if (!inputElement) {
        // 策略3: 根据 placeholder 类型使用不同的备用选择器
        if (placeholder.includes('题干')) {
            // 题干的备用选择器
            inputElement = container.querySelector('.ckeditor_div[contenteditable="true"]') ||
                          container.querySelector('[contenteditable="true"].ckeditor_div') ||
                          container.querySelector('.question-stem [contenteditable="true"]');
        } else if (placeholder.includes('解析')) {
            // 解析的备用选择器
            inputElement = container.querySelector('.analysis [contenteditable="true"]') ||
                          container.querySelector('.explanation [contenteditable="true"]') ||
                          Array.from(container.querySelectorAll('[contenteditable="true"]')).find(el => 
                              el.getAttribute('placeholder') && el.getAttribute('placeholder').includes('解析')
                          );
        }
    }
    
    if (!inputElement) {
        // 策略4: 全局查找（作为最后手段）
        console.log(`🔍 在全局范围内查找 "${placeholder}" 的输入框...`);
        selector = `[contenteditable="true"][placeholder*="${placeholder}"]`;
        inputElement = document.querySelector(selector);
    }

    if (inputElement) {
        console.log(`🎯 找到输入框:`, inputElement);
        inputElement.classList.remove('placeholder'); // 移除占位符样式
        inputElement.innerHTML = `<p>${text}</p>`;    // 填入内容
        triggerEvents(inputElement);                   // 触发事件
        console.log(`✅ 成功填充 "${placeholder}"`);
    } else {
        console.warn(`⚠️ 填充 "${placeholder}" 失败: 找不到对应的输入框`);
        // 调试信息：列出容器内所有可编辑元素
        const allEditableElements = container.querySelectorAll('[contenteditable="true"]');
        console.log(`📋 容器内找到 ${allEditableElements.length} 个可编辑元素:`);
        allEditableElements.forEach((el, index) => {
            console.log(`  ${index + 1}. placeholder: "${el.getAttribute('placeholder')}", class: "${el.className}"`);
        });
    }
    await delay(100); // 每个填充操作后短暂延时，增加稳定性
}
// 填充题目内容的函数
async function fillQuestionContent(questionData) {
    console.log('开始填充题目内容');

    // 等待页面加载
    await delay(800);

    // 找到当前活动的题目表单容器
    let currentForm = document.querySelector('.question-item.active');
    if (!currentForm) {
        // 备用选择器：查找最后一个题目容器或当前编辑的题目
        const allQuestions = document.querySelectorAll('.question-item');
        if (allQuestions.length > 0) {
            currentForm = allQuestions[allQuestions.length - 1];
        }
    }
    if (!currentForm) {
        // 最后的备用选择器：查找包含编辑表单的容器
        currentForm = document.querySelector('.question-form') || 
                     document.querySelector('.question-content') ||
                     document.querySelector('.form-container') ||
                     document;
    }

    console.log('🎯 当前题目表单容器:', currentForm);
    console.log('📊 容器类名:', currentForm.className);
    
    // 调试：列出容器内所有可编辑元素
    const allEditableInContainer = currentForm.querySelectorAll('[contenteditable="true"]');
    console.log(`📋 容器内共找到 ${allEditableInContainer.length} 个可编辑元素`);

    // 步骤 3: 填充题干
    await fillEditableDiv(currentForm, '请录入题干', questionData.stem);

    // 等待内容保存
    await delay(300);

    // 步骤 4: 填充选项
    var optionInputs = currentForm.querySelectorAll('.options .ckeditor_div[contenteditable="true"]');
    if (optionInputs.length === 0) {
        // 备用选择器
        optionInputs = document.querySelectorAll('.options .ckeditor_div[contenteditable="true"]');
    }

    for (let i = 0; i < questionData.options.length; i++) {
        if (optionInputs[i]) {
            optionInputs[i].classList.remove('placeholder');
            optionInputs[i].innerHTML = questionData.options[i];
            triggerEvents(optionInputs[i]);
            console.log(`✅ 成功设置选项 ${String.fromCharCode(65 + i)}: ${questionData.options[i]}`);
        } else {
            console.warn(`⚠️ 找不到选项 ${String.fromCharCode(65 + i)} 的输入框`);
        }
        await delay(100); // 每个操作间短暂延时
    }

    // 步骤 5: 设置答案 (根据索引)
    var radioButtons = currentForm.querySelectorAll('.ant-radio-group input[type="radio"]');
    if (radioButtons.length === 0) {
        radioButtons = document.querySelectorAll('.ant-radio-group input[type="radio"]');
    }

    if (radioButtons[questionData.answer]) {
        radioButtons[questionData.answer].click();
        console.log(`✅ 成功设置答案: ${String.fromCharCode(65 + questionData.answer)}`);
    } else {
        console.warn(`⚠️ 找不到索引为 ${questionData.answer} 的答案单选按钮`);
    }
    await delay(100);

    // 步骤 6: 填充解析
    await fillEditableDiv(currentForm, '请录入解析', questionData.analysis);

    // 点击保存按钮
    var saveButton = document.querySelector('button.ant-btn.ant-btn-primary[data-v-4c71fb2d]');
    if (!saveButton) {
        // 备用选择器
        saveButton = document.querySelector('button.ant-btn.ant-btn-primary');
        if (!saveButton) {
            saveButton = Array.from(document.querySelectorAll('button')).find(btn =>
                btn.textContent.includes('保存') || btn.textContent.includes('保 存')
            );
        }
    }

    if (saveButton) {
        saveButton.click();
        console.log('✅ 已点击保存按钮');
        await delay(1000);
    } else {
        console.error('❌ 未找到保存按钮');
    }

    // 等待一下让内容保存
    await delay(500);
    console.log('题目内容填充完成');
}


/**
 * 触发一个元素上的多个事件，以模拟真实用户操作，确保框架能接收到变更
 * @param {HTMLElement} element - 目标元素
 */
function triggerEvents(element) {
    element.focus();
    // 触发一系列事件，确保兼容各种前端框架
    ['input', 'change', 'keyup', 'blur'].forEach(eventType => {
        element.dispatchEvent(new Event(eventType, { bubbles: true, cancelable: true }));
    });
}

/**
 * 模拟键盘输入到可编辑元素
 * @param {HTMLElement} element - 目标元素
 * @param {string} content - 要输入的内容（支持HTML）
 */
async function simulateContentInput(element, content) {
    if (!element) {
        console.warn('⚠️ 目标元素不存在，跳过填充');
        return;
    }

    element.focus();

    // 触发开始编辑事件
    element.dispatchEvent(new KeyboardEvent('keydown', { bubbles: true }));

    // 设置内容
    element.innerHTML = content;

    // 触发一系列输入相关事件
    const events = ['input', 'textInput', 'keyup', 'change'];
    events.forEach(eventType => {
        element.dispatchEvent(new Event(eventType, { bubbles: true, cancelable: true }));
    });

    // 触发结束编辑事件
    element.dispatchEvent(new Event('blur', { bubbles: true }));

    console.log("✅ 模拟键盘输入完成");

    // 短暂延时确保内容稳定
    await new Promise(resolve => setTimeout(resolve, 100));
}

/**
 * 触发元素事件，确保页面能识别到内容变化（优化版本）
 * @param {HTMLElement} element - 目标元素
 */
function triggerInputEvents(element) {
    if (!element) return;

    element.focus();
    ['input', 'change', 'keyup', 'blur'].forEach(eventType => {
        element.dispatchEvent(new Event(eventType, { bubbles: true, cancelable: true }));
    });
}

// 定位并点击最后一题的函数
async function locateAndClickLastQuestion() {
    // 查找所有题目容器
    var allQuestions = document.querySelectorAll('.question-item');

    if (allQuestions.length > 0) {
        // 获取最后一个题目
        var lastQuestion = allQuestions[allQuestions.length - 1];

        // 滚动到最后一题
        lastQuestion.scrollIntoView({ behavior: 'smooth', block: 'center' });

        // 点击最后一题
        lastQuestion.click();

        console.log('已点击最后一题，ID:', lastQuestion.id);

        // 等待一下让页面响应
        await new Promise(resolve => setTimeout(resolve, 500));

        return true;
    } else {
        console.log('未找到任何题目');
        return false;
    }
}

// 添加新题目的函数
async function addNewQuestion() {
    // 查找"添加题目"按钮 - 多种选择器
    var addButton = document.querySelectorAll('.add-operate-item')[1];

    if (!addButton) {
        // 备用选择器1：通过文本内容查找
        addButton = Array.from(document.querySelectorAll('button, .add-operate-item')).find(btn =>
            btn.textContent && btn.textContent.includes('添加题目')
        );
    }

    if (!addButton) {
        // 备用选择器2：通过类名查找
        addButton = document.querySelector('.add-operate-item');
    }

    if (addButton) {
        // 点击添加题目按钮
        addButton.click();
        console.log('✅ 已点击添加题目按钮');

        // 等待新题目创建完成
        await delay(1500); // 增加等待时间，确保题目完全创建
        return true;
    } else {
        console.warn('⚠️ 未找到添加题目按钮，可能已在编辑状态');
        return false;
    }
}

// 主执行函数
async function main() {
    try {
        console.log(`🚀 脚本启动，准备处理 ${Questions.length} 道单选题...`);

        for (let i = 0; i < Questions.length; i++) {
            const questionData = Questions[i];
            console.log(`\n--- [ ${i + 1} / ${Questions.length} ] --- 开始处理第 ${i + 1} 个题目`);

            // 1. 先定位并点击最后一题
            const locateSuccess = await locateAndClickLastQuestion();
            if (!locateSuccess) {
                console.error(`第 ${i + 1} 个题目：无法定位到最后一题`);
                continue;
            }

            // 2. 添加新题目（如果不是第一题）
            const addSuccess = await addNewQuestion();
            if (!addSuccess) {
                console.error(`第 ${i + 1} 个题目：无法添加新题目`);
                continue;
            }

            // 3. 再次定位到新创建的最后一题
            await locateAndClickLastQuestion();


            // 4. 设置题型为单选题
            const typeSetSuccess = await operateElements();
            if (!typeSetSuccess) {
                console.warn(`第 ${i + 1} 个题目：题型设置可能失败，继续尝试填充内容`);
            }





            // // 获取所有选项关闭按钮（X）并删除第一个
            // const optionCloseButtons = document.querySelectorAll('.options-close');
            // if (optionCloseButtons.length > 0) {
            //     optionCloseButtons[0].click();
            //     console.log('✅ 已点击第一个选项关闭按钮');
            //     await delay(300);
            // } else {
            //     console.warn('⚠️ 未找到选项关闭按钮');
            // }


            // 5. 填充题目内容
            await fillQuestionContent(questionData);

            console.log(`✅ 第 ${i + 1} 个题目处理完成`);

            // 每个题目之间稍作停顿
            await delay(1000);
        }

        console.log('\n🎉🎉🎉 所有题目处理完成！');
    } catch (error) {
        console.error('💥 执行过程中出现错误:', error);
        console.error('请检查页面结构或刷新页面后重试。');
    }
}

// 执行主函数
main();

    "#,
        )
    }

    /// 阅读理解附加代码  
    fn get_reading_code(&self) -> String {
        String::from(
            r#"

//MARK： 使用XPath查找包含"阅读理解"文本的元素
function clickReadingElement() {
    // XPath表达式：查找class包含"tag"且包含"阅读理解"文本的元素
    var xpath = "//*[contains(@class,'tag') and contains(text(),'阅读理解')]";

    // 执行XPath查询
    var result = document.evaluate(
        xpath,
        document,
        null,
        XPathResult.FIRST_ORDERED_NODE_TYPE,
        null
    );

    // 如果找到元素，点击它
    if (result.singleNodeValue) {
        result.singleNodeValue.click();
        console.log("成功点击阅读理解元素");
        return true;
    } else {
        console.log("未找到包含'阅读理解'文本的元素");
        return false;
    }
}

// 完整的操作流程
async function operateElements() {
    // 1. 点击下拉框
    var selectDiv = document.querySelector('div[title="单选题"]');
    if (selectDiv) {
        selectDiv.click();

        // 2. 选择复合题 - 使用 Promise 替代 setTimeout
        await new Promise(resolve => {
            setTimeout(function () {
                var options = document.querySelectorAll('li.ant-select-dropdown-menu-item');
                for (var i = 0; i < options.length; i++) {
                    if (options[i].textContent.trim() === '复合题') {
                        options[i].click();
                        break;
                    }
                }
                resolve();
            }, 100);
        });

        // 3. 使用XPath点击阅读理解标签 - 使用 Promise 替代 setTimeout
        await new Promise(resolve => {
            setTimeout(function () {
                clickReadingElement();
                resolve();
            }, 200);
        });
    }
}



/**
 * 模拟键盘输入到可编辑元素
 * @param {HTMLElement} element - 目标元素
 * @param {string} content - 要输入的内容（支持HTML）
 */
async function simulateContentInput(element, content) {
    element.focus();

    // 触发开始编辑事件
    element.dispatchEvent(new KeyboardEvent('keydown', { bubbles: true }));

    // 设置内容
    element.innerHTML = content;

    // 触发一系列输入相关事件
    const events = ['input', 'textInput', 'keyup', 'change'];
    events.forEach(eventType => {
        element.dispatchEvent(new Event(eventType, { bubbles: true, cancelable: true }));
    });

    // 触发结束编辑事件
    element.dispatchEvent(new Event('blur', { bubbles: true }));

    console.log("✅ 模拟键盘输入完成");
}

/**
 * 使用模拟键盘输入设置初始内容
 */
async function setInitialContent() {
    console.log("📝 开始模拟键盘输入设置初始内容...");

    const showBoxElement = document.querySelector('.showBox');
    const ckeditorElement = document.querySelector('.ckeditor_div.cke_editable');

    if (showBoxElement) {
        await simulateContentInput(showBoxElement, newContent);
    }

    if (ckeditorElement) {
        await simulateContentInput(ckeditorElement, newContent);
    }

    await delay(500); // 等待内容稳定
}

/**
 * 等待指定毫秒数
 * @param {number} ms - 等待的时间（毫秒）
 */
var delay = (ms) => new Promise(resolve => setTimeout(resolve, ms));

/**
 * 触发一个元素上的多个事件，以模拟真实用户操作，确保框架能接收到变更
 * @param {HTMLElement} element - 目标元素
 */
function triggerEvents(element) {
    element.focus();
    // 触发一系列事件，确保兼容各种前端框架
    ['input', 'change', 'keyup', 'blur'].forEach(eventType => {
        element.dispatchEvent(new Event(eventType, { bubbles: true, cancelable: true }));
    });
}

/**
 * 封装好的填充函数，用于向可编辑的 div 填入内容
 * @param {HTMLElement} container - 题目总容器
 * @param {string} placeholder - 通过 placeholder 文本来精确定位输入框
 * @param {string} text - 要填充的 HTML 内容
 */
async function fillEditableDiv(container, placeholder, text) {
    const selector = `[contenteditable="true"][placeholder="${placeholder}"]`;
    const inputElement = container.querySelector(selector);

    if (inputElement) {
        inputElement.classList.remove('placeholder'); // 移除占位符样式
        inputElement.innerHTML = `<p>${text}</p>`;    // 填入内容
        triggerEvents(inputElement);                   // 触发事件
        console.log(`✅ 成功填充 "${placeholder}"`);
    } else {
        console.warn(`⚠️ 填充 "${placeholder}" 失败: 找不到对应的输入框`);
    }
    await delay(100); // 每个填充操作后短暂延时，增加稳定性
}


// ----------- 3. 主执行函数 (简单直接的核心流程) -----------

async function processAllQuestions() {

    console.log(`Switch to 复合题/阅读理解 mode...`);
    await operateElements();
    console.log(`🚀 脚本启动，插入题目文章`);
    document.querySelector('.showBox').innerHTML = newContent;
    document.querySelector('.ckeditor_div.cke_editable').innerHTML = newContent;

    console.log(`🚀 脚本启动，准备处理 ${Questions.length} 道题目...`);
    try {
        // 先用模拟键盘输入设置初始内容
        await setInitialContent();

        for (const [index, questionData] of Questions.entries()) {
            console.log(`\n--- [ ${index + 1} / ${Questions.length} ] --- 开始处理新题目...`);

            // 步骤 1: 点击 "添加小题" -> 选择 "单选题" -> 点击 "确定"
            const addSubQuestionButton = Array.from(document.querySelectorAll('button.add-fuhuxiao-btn span')).find(el => el.textContent.trim() === '添加小题');
            if (!addSubQuestionButton) throw new Error("找不到 '添加小题' 按钮！");
            addSubQuestionButton.parentElement.click();
            await delay(500);

            const singleChoiceOption = Array.from(document.querySelectorAll('.add-fuhuxiao-content .form-value-span')).find(el => el.textContent.trim() === '单选题');
            if (!singleChoiceOption) throw new Error("在弹窗中找不到 '单选题' 选项！");
            singleChoiceOption.click();

            const confirmButton = Array.from(document.querySelectorAll('.add-fuhuxiao-footer button span')).find(el => el.textContent.trim() === '确 定');
            if (!confirmButton) throw new Error("在弹窗中找不到 '确定' 按钮！");
            confirmButton.parentElement.click();

            console.log("🌀 已创建新小题，等待表单完全加载...");
            await delay(1500); // **关键延时**: 等待新题目表单渲染

            // 步骤 2: 定位到最新添加的题目容器 (总是最后一个)
            const allForms = document.querySelectorAll('.fuhe-content-wrap');
            const currentForm = allForms[allForms.length - 1];
            if (!currentForm) throw new Error("找不到新创建的小题表单容器！");

            // 步骤 3: 填充题干
            await fillEditableDiv(currentForm, '请录入小题题干', questionData.stem);

            // 步骤 4: 填充选项
            var optionInputs = currentForm.querySelectorAll('.options .ckeditor_div[contenteditable="true"]');
            for (let i = 0; i < questionData.options.length; i++) {
                if (optionInputs[i]) {
                    optionInputs[i].classList.remove('placeholder');
                    optionInputs[i].innerHTML = questionData.options[i];
                    triggerEvents(optionInputs[i]);
                    console.log(`✅ 成功设置选项 ${String.fromCharCode(65 + i)}: ${questionData.options[i]}`);
                } else {
                    console.warn(`⚠️ 找不到选项 ${String.fromCharCode(65 + i)} 的输入框`);
                }
                await delay(100); // 每个操作间短暂延时
            }

            // 步骤 5: 设置答案 (根据索引)
            var radioButtons = currentForm.querySelectorAll('.ant-radio-group input[type="radio"]');
            if (radioButtons[questionData.answer]) {
                radioButtons[questionData.answer].click();
                console.log(`✅ 成功设置答案: ${String.fromCharCode(65 + questionData.answer)}`);
            } else {
                console.warn(`⚠️ 找不到索引为 ${questionData.answer} 的答案单选按钮`);
            }
            await delay(100);

            // 步骤 6: 填充解析
            await fillEditableDiv(currentForm, '请录入解析', questionData.analysis);

            console.log(`👍 第 ${index + 1} 题处理完成！`);
        }

        console.log("\n🎉🎉🎉 所有题目均已成功处理！");

    } catch (error) {
        console.error("💥 脚本执行过程中发生严重错误:", error);
        console.error("请检查页面结构或刷新页面后重试。");
    }
}

// 启动脚本
processAllQuestions();"#,
        )
    }

    /// 完形填空附加代码
    fn get_cloze_test_code(&self) -> String {
        String::from(
            r#"/**
 * 模拟键盘输入到可编辑元素
 * @param {HTMLElement} element - 目标元素
 * @param {string} content - 要输入的内容（支持HTML）
 */
async function simulateContentInput(element, content) {
    element.focus();

    // 触发开始编辑事件
    element.dispatchEvent(new KeyboardEvent('keydown', { bubbles: true }));

    // 设置内容
    element.innerHTML = content;

    // 触发一系列输入相关事件
    const events = ['input', 'textInput', 'keyup', 'change'];
    events.forEach(eventType => {
        element.dispatchEvent(new Event(eventType, { bubbles: true, cancelable: true }));
    });

    // 触发结束编辑事件
    element.dispatchEvent(new Event('blur', { bubbles: true }));

    console.log("✅ 模拟键盘输入完成");
}

/**
 * 使用模拟键盘输入设置初始内容
 */
async function setInitialContent() {
    console.log("📝 开始模拟键盘输入设置初始内容...");

    const showBoxElement = document.querySelector('.showBox');
    const ckeditorElement = document.querySelector('.ckeditor_div.cke_editable');

    if (showBoxElement) {
        await simulateContentInput(showBoxElement, newContent);
    }

    if (ckeditorElement) {
        await simulateContentInput(ckeditorElement, newContent);
    }

    await delay(500); // 等待内容稳定
}

/**
 * 等待指定毫秒数
 * @param {number} ms - 等待的时间（毫秒）
 */
var delay = (ms) => new Promise(resolve => setTimeout(resolve, ms));

/**
 * 触发一个元素上的多个事件，以模拟真实用户操作，确保框架能接收到变更
 * @param {HTMLElement} element - 目标元素
 */
function triggerEvents(element) {
    element.focus();
    // 触发一系列事件，确保兼容各种前端框架
    ['input', 'change', 'keyup', 'blur'].forEach(eventType => {
        element.dispatchEvent(new Event(eventType, { bubbles: true, cancelable: true }));
    });
}

/**
 * 封装好的填充函数，用于向可编辑的 div 填入内容
 * @param {HTMLElement} container - 题目总容器
 * @param {string} placeholder - 通过 placeholder 文本来精确定位输入框
 * @param {string} text - 要填充的 HTML 内容
 */
async function fillEditableDiv(container, placeholder, text) {
    const selector = `[contenteditable="true"][placeholder="${placeholder}"]`;
    const inputElement = container.querySelector(selector);

    if (inputElement) {
        inputElement.classList.remove('placeholder'); // 移除占位符样式
        inputElement.innerHTML = `<p>${text}</p>`;    // 填入内容
        triggerEvents(inputElement);                   // 触发事件
        console.log(`✅ 成功填充 "${placeholder}"`);
    } else {
        console.warn(`⚠️ 填充 "${placeholder}" 失败: 找不到对应的输入框`);
    }
    await delay(100); // 每个填充操作后短暂延时，增加稳定性
}

// ----------- 完形填空题目配置功能 -----------

/**
 * 配置单个完形填空题目
 * @param {number} questionIndex - 题目索引
 * @param {Object} questionData - 题目数据
 */
async function configureQuestion(questionIndex, questionData) {
    console.log(`\n--- [ ${questionIndex + 1} / ${Questions.length} ] --- 开始配置题目...`);

    try {
        // 步骤 1: 点击对应的空格标签
        const blankTabs = document.querySelectorAll('.blank-name');
        if (!blankTabs[questionIndex]) {
            throw new Error(`找不到第${questionIndex + 1}题的标签`);
        }

        blankTabs[questionIndex].click();
        await delay(500); // 等待标签切换

        // 步骤 2: 找到当前显示的配置区域
        const activeConfig = document.querySelector('.blank-config-item:not([style*="display: none"])');
        if (!activeConfig) {
            throw new Error(`找不到第${questionIndex + 1}题的配置区域`);
        }

        // 步骤 3: 填充选项A、B、C、D
        console.log(`正在配置第${questionIndex + 1}题的选项...`);
        const optionInputs = activeConfig.querySelectorAll('.options .ckeditor_div[contenteditable="true"]');

        for (let i = 0; i < questionData.options.length && i < optionInputs.length; i++) {
            if (optionInputs[i]) {
                optionInputs[i].classList.remove('placeholder');
                optionInputs[i].innerHTML = questionData.options[i];
                triggerEvents(optionInputs[i]);
                console.log(`✅ 成功设置选项 ${String.fromCharCode(65 + i)}: ${questionData.options[i]}`);
            }
            await delay(100);
        }

        // 步骤 4: 设置答案
        console.log(`设置答案: ${String.fromCharCode(65 + questionData.answer)}`);
        const radioButtons = activeConfig.querySelectorAll('.ant-radio-group input[type="radio"]');
        if (radioButtons[questionData.answer]) {
            radioButtons[questionData.answer].click();
            console.log(`✅ 成功设置答案: ${String.fromCharCode(65 + questionData.answer)}`);
        } else {
            console.warn(`⚠️ 找不到索引为 ${questionData.answer} 的答案单选按钮`);
        }
        await delay(100);

        // 步骤 5: 填充解析
        console.log(`开始输入解析...`);
        let explanationInput = activeConfig.querySelector('[placeholder="请录入解析"][contenteditable="true"]');

        if (!explanationInput) {
            explanationInput = activeConfig.querySelector('.ckeditor_div[placeholder="请录入解析"]');
        }

        if (explanationInput) {
            explanationInput.classList.remove('placeholder');
            explanationInput.innerHTML = questionData.analysis;
            triggerEvents(explanationInput);
            console.log(`✅ 成功填充解析`);
        } else {
            console.warn(`⚠️ 解析输入框未找到`);
        }

        console.log(`👍 第 ${questionIndex + 1} 题配置完成！`);

    } catch (error) {
        console.error(`💥 配置第${questionIndex + 1}题时发生错误:`, error);
    }
}

// ----------- 主执行函数 -----------

/**
 * 处理所有完形填空题目
 */
async function processAllQuestions() {
    console.log(`🚀 完形填空配置脚本启动，准备处理 ${Questions.length} 道题目...`);

    try {
        // 步骤 1: 设置文章内容
        console.log("📝 设置完形填空文章内容...");
        await setInitialContent();

        // 步骤 2: 逐个配置题目
        for (const [index, questionData] of Questions.entries()) {
            await configureQuestion(index, questionData);
            await delay(500); // 题目间延时
        }

        console.log("\n🎉🎉🎉 所有题目均已成功配置！");

    } catch (error) {
        console.error("💥 脚本执行过程中发生严重错误:", error);
        console.error("请检查页面结构或刷新页面后重试。");
    }
}

// 启动脚本
processAllQuestions();

// 导出函数供手动调用
console.log("🎉 完形填空一键配置脚本已加载！");
console.log("脚本功能：1. 文章内容设置 -> 2. 题目选项配置 -> 3. 答案设置 -> 4. 解析输入");
console.log("可用函数：");
console.log("- processAllQuestions()：重新执行完整配置");
console.log("- configureQuestion(index, data)：配置单个题目");
console.log("- setInitialContent()：仅设置文章内容");

// 挂载到window对象
window.processAllQuestions = processAllQuestions;
window.configureQuestion = configureQuestion;
window.setInitialContent = setInitialContent;"#,
        )
    }

    fn get_listening_compound_code(&self) -> String {
        String::from(
            r#"
//MARK： 使用XPath查找包含"阅读理解"文本的元素
function clickReadingElement() {
    // XPath表达式：查找class包含"tag"且包含"阅读理解"文本的元素
    var xpath = "//*[contains(@class,'tag') and contains(text(),'阅读理解')]";

    // 执行XPath查询
    var result = document.evaluate(
        xpath,
        document,
        null,
        XPathResult.FIRST_ORDERED_NODE_TYPE,
        null
    );

    // 如果找到元素，点击它
    if (result.singleNodeValue) {
        result.singleNodeValue.click();
        console.log("成功点击阅读理解元素");
        return true;
    } else {
        console.log("未找到包含'阅读理解'文本的元素");
        return false;
    }
}

// 完整的操作流程
async function operateElements() {
    // 1. 点击下拉框
    var selectDiv = document.querySelector('div[title="单选题"]');
    if (selectDiv) {
        selectDiv.click();

        // 2. 选择复合题 - 使用 Promise 替代 setTimeout
        await new Promise(resolve => {
            setTimeout(function () {
                var options = document.querySelectorAll('li.ant-select-dropdown-menu-item');
                for (var i = 0; i < options.length; i++) {
                    if (options[i].textContent.trim() === '复合题') {
                        options[i].click();
                        break;
                    }
                }
                resolve();
            }, 100);
        });

        // 3. 使用XPath点击阅读理解标签 - 使用 Promise 替代 setTimeout
        await new Promise(resolve => {
            setTimeout(function () {
                clickReadingElement();
                resolve();
            }, 200);
        });
    }
}



/**
 * 模拟键盘输入到可编辑元素
 * @param {HTMLElement} element - 目标元素
 * @param {string} content - 要输入的内容（支持HTML）
 */
async function simulateContentInput(element, content) {
    element.focus();

    // 触发开始编辑事件
    element.dispatchEvent(new KeyboardEvent('keydown', { bubbles: true }));

    // 设置内容
    element.innerHTML = content;

    // 触发一系列输入相关事件
    const events = ['input', 'textInput', 'keyup', 'change'];
    events.forEach(eventType => {
        element.dispatchEvent(new Event(eventType, { bubbles: true, cancelable: true }));
    });

    // 触发结束编辑事件
    element.dispatchEvent(new Event('blur', { bubbles: true }));

    console.log("✅ 模拟键盘输入完成");
}

/**
 * 使用模拟键盘输入设置初始内容
 */
async function setInitialContent() {
    console.log("📝 开始模拟键盘输入设置初始内容...");

    const showBoxElement = document.querySelector('.showBox');
    const ckeditorElement = document.querySelector('.ckeditor_div.cke_editable');

    if (showBoxElement) {
        await simulateContentInput(showBoxElement, newContent);
    }

    if (ckeditorElement) {
        await simulateContentInput(ckeditorElement, newContent);
    }

    await delay(500); // 等待内容稳定
}

/**
 * 等待指定毫秒数
 * @param {number} ms - 等待的时间（毫秒）
 */
var delay = (ms) => new Promise(resolve => setTimeout(resolve, ms));

/**
 * 触发一个元素上的多个事件，以模拟真实用户操作，确保框架能接收到变更
 * @param {HTMLElement} element - 目标元素
 */
function triggerEvents(element) {
    element.focus();
    // 触发一系列事件，确保兼容各种前端框架
    ['input', 'change', 'keyup', 'blur'].forEach(eventType => {
        element.dispatchEvent(new Event(eventType, { bubbles: true, cancelable: true }));
    });
}

/**
 * 封装好的填充函数，用于向可编辑的 div 填入内容
 * @param {HTMLElement} container - 题目总容器
 * @param {string} placeholder - 通过 placeholder 文本来精确定位输入框
 * @param {string} text - 要填充的 HTML 内容
 */
async function fillEditableDiv(container, placeholder, text) {
    const selector = `[contenteditable="true"][placeholder="${placeholder}"]`;
    const inputElement = container.querySelector(selector);

    if (inputElement) {
        inputElement.classList.remove('placeholder'); // 移除占位符样式
        inputElement.innerHTML = `<p>${text}</p>`;    // 填入内容
        triggerEvents(inputElement);                   // 触发事件
        console.log(`✅ 成功填充 "${placeholder}"`);
    } else {
        console.warn(`⚠️ 填充 "${placeholder}" 失败: 找不到对应的输入框`);
    }
    await delay(100); // 每个填充操作后短暂延时，增加稳定性
}


// ----------- 3. 主执行函数 (简单直接的核心流程) -----------

async function processAllQuestions() {

    console.log(`Switch to 复合题/阅读理解 mode...`);
    await operateElements();
    console.log(`🚀 脚本启动，插入题目文章`);
    document.querySelector('.showBox').innerHTML = newContent;
    document.querySelector('.ckeditor_div.cke_editable').innerHTML = newContent;

    console.log(`🚀 脚本启动，准备处理 ${Questions.length} 道题目...`);
    try {
        // 先用模拟键盘输入设置初始内容
        await setInitialContent();

        for (const [index, questionData] of Questions.entries()) {
            console.log(`\n--- [ ${index + 1} / ${Questions.length} ] --- 开始处理新题目...`);

            // 步骤 1: 点击 "添加小题" -> 选择 "单选题" -> 点击 "确定"
            const addSubQuestionButton = Array.from(document.querySelectorAll('button.add-fuhuxiao-btn span')).find(el => el.textContent.trim() === '添加小题');
            if (!addSubQuestionButton) throw new Error("找不到 '添加小题' 按钮！");
            addSubQuestionButton.parentElement.click();
            await delay(500);

            const singleChoiceOption = Array.from(document.querySelectorAll('.add-fuhuxiao-content .form-value-span')).find(el => el.textContent.trim() === '单选题');
            if (!singleChoiceOption) throw new Error("在弹窗中找不到 '单选题' 选项！");
            singleChoiceOption.click();

            const confirmButton = Array.from(document.querySelectorAll('.add-fuhuxiao-footer button span')).find(el => el.textContent.trim() === '确 定');
            if (!confirmButton) throw new Error("在弹窗中找不到 '确定' 按钮！");
            confirmButton.parentElement.click();

            console.log("🌀 已创建新小题，等待表单完全加载...");
            await delay(1500); // **关键延时**: 等待新题目表单渲染

            // 步骤 2: 定位到最新添加的题目容器 (总是最后一个)
            const allForms = document.querySelectorAll('.fuhe-content-wrap');
            const currentForm = allForms[allForms.length - 1];
            if (!currentForm) throw new Error("找不到新创建的小题表单容器！");

            // 步骤 3: 填充题干
            await fillEditableDiv(currentForm, '请录入小题题干', questionData.stem);

            // 步骤 4: 填充选项
            var optionInputs = currentForm.querySelectorAll('.options .ckeditor_div[contenteditable="true"]');
            for (let i = 0; i < questionData.options.length; i++) {
                if (optionInputs[i]) {
                    optionInputs[i].classList.remove('placeholder');
                    optionInputs[i].innerHTML = questionData.options[i];
                    triggerEvents(optionInputs[i]);
                    console.log(`✅ 成功设置选项 ${String.fromCharCode(65 + i)}: ${questionData.options[i]}`);
                } else {
                    console.warn(`⚠️ 找不到选项 ${String.fromCharCode(65 + i)} 的输入框`);
                }
                await delay(100); // 每个操作间短暂延时
            }

            // 步骤 5: 设置答案 (根据索引)
            var radioButtons = currentForm.querySelectorAll('.ant-radio-group input[type="radio"]');
            if (radioButtons[questionData.answer]) {
                radioButtons[questionData.answer].click();
                console.log(`✅ 成功设置答案: ${String.fromCharCode(65 + questionData.answer)}`);
            } else {
                console.warn(`⚠️ 找不到索引为 ${questionData.answer} 的答案单选按钮`);
            }
            await delay(100);

            // 步骤 6: 填充解析
            await fillEditableDiv(currentForm, '请录入解析', questionData.analysis);

            console.log(`👍 第 ${index + 1} 题处理完成！`);
        }

        console.log("\n🎉🎉🎉 所有题目均已成功处理！");

    } catch (error) {
        console.error("💥 脚本执行过程中发生严重错误:", error);
        console.error("请检查页面结构或刷新页面后重试。");
    }
}

// 启动脚本
processAllQuestions();
"#,
        )
    }

    fn get_listening_single_code(&self) -> String {
        String::from(
            r#"
//MARK： 使用XPath查找包含指定文本的元素
var delay = (ms) => new Promise(resolve => setTimeout(resolve, ms));
function clickBlankFillingElement(type) {
    // XPath表达式：查找class包含"tag"且包含指定文本的元素
    var xpath = "//*[contains(@class,'tag') and contains(text(),'" + type + "')]";

    // 执行XPath查询
    var result = document.evaluate(
        xpath,
        document,
        null,
        XPathResult.FIRST_ORDERED_NODE_TYPE,
        null
    );

    // 如果找到元素，点击它
    if (result.singleNodeValue) {
        result.singleNodeValue.click();
        console.log(`成功点击 ${type} 标签元素`);
        return true;
    } else {
        console.log(`未找到包含 '${type}' 文本的标签元素`);
        return false;
    }
}
//
// 完整的操作流程 - 设置为单选题
async function operateElements() {
    console.log("开始设置题型为单选题...");

    // 1. 点击下拉框 - 查找当前选中的题型
    var selectDiv = document.querySelector('div[title]');
    if (!selectDiv) {
        // 备用选择器
        selectDiv = document.querySelector('.ant-select-selection-selected-value');
        if (!selectDiv) {
            selectDiv = document.querySelector('.ant-select-selection__rendered');
        }
    }

    if (selectDiv) {
        selectDiv.click();
        console.log("✅ 已点击题型下拉框");

        // 2. 等待下拉菜单出现，然后选择单选题
        await new Promise(resolve => {
            setTimeout(function () {
                var options = document.querySelectorAll('li.ant-select-dropdown-menu-item');
                for (var i = 0; i < options.length; i++) {
                    if (options[i].textContent.trim() === '单选题') {
                        options[i].click();
                        console.log("✅ 已选择单选题");
                        break;
                    }
                }
                resolve();
            }, 200);
        });

        // 3. 等待一下确保选择生效
        await new Promise(resolve => setTimeout(resolve, 300));

        console.log("✅ 题型设置完成");
        return true;
    } else {
        console.error("❌ 未找到题型下拉框");
        return false;
    }
}
/**
 * 封装好的填充函数，用于向可编辑的 div 填入内容
 * @param {HTMLElement} container - 题目总容器
 * @param {string} placeholder - 通过 placeholder 文本来精确定位输入框
 * @param {string} text - 要填充的 HTML 内容
 */
async function fillEditableDiv(container, placeholder, text) {
    // 多种选择器策略
    let inputElement = null;
    
    // 策略1: 精确匹配 placeholder
    let selector = `[contenteditable="true"][placeholder="${placeholder}"]`;
    inputElement = container.querySelector(selector);
    
    if (!inputElement) {
        // 策略2: 查找包含 placeholder 文本的元素
        selector = `[contenteditable="true"]`;
        const editableElements = container.querySelectorAll(selector);
        for (let element of editableElements) {
            if (element.getAttribute('placeholder') && element.getAttribute('placeholder').includes(placeholder)) {
                inputElement = element;
                break;
            }
        }
    }
    
    if (!inputElement) {
        // 策略3: 根据 placeholder 类型使用不同的备用选择器
        if (placeholder.includes('题干')) {
            // 题干的备用选择器
            inputElement = container.querySelector('.ckeditor_div[contenteditable="true"]') ||
                          container.querySelector('[contenteditable="true"].ckeditor_div') ||
                          container.querySelector('.question-stem [contenteditable="true"]');
        } else if (placeholder.includes('解析')) {
            // 解析的备用选择器
            inputElement = container.querySelector('.analysis [contenteditable="true"]') ||
                          container.querySelector('.explanation [contenteditable="true"]') ||
                          Array.from(container.querySelectorAll('[contenteditable="true"]')).find(el => 
                              el.getAttribute('placeholder') && el.getAttribute('placeholder').includes('解析')
                          );
        }
    }
    
    if (!inputElement) {
        // 策略4: 全局查找（作为最后手段）
        console.log(`🔍 在全局范围内查找 "${placeholder}" 的输入框...`);
        selector = `[contenteditable="true"][placeholder*="${placeholder}"]`;
        inputElement = document.querySelector(selector);
    }

    if (inputElement) {
        console.log(`🎯 找到输入框:`, inputElement);
        inputElement.classList.remove('placeholder'); // 移除占位符样式
        inputElement.innerHTML = `<p>${text}</p>`;    // 填入内容
        triggerEvents(inputElement);                   // 触发事件
        console.log(`✅ 成功填充 "${placeholder}"`);
    } else {
        console.warn(`⚠️ 填充 "${placeholder}" 失败: 找不到对应的输入框`);
        // 调试信息：列出容器内所有可编辑元素
        const allEditableElements = container.querySelectorAll('[contenteditable="true"]');
        console.log(`📋 容器内找到 ${allEditableElements.length} 个可编辑元素:`);
        allEditableElements.forEach((el, index) => {
            console.log(`  ${index + 1}. placeholder: "${el.getAttribute('placeholder')}", class: "${el.className}"`);
        });
    }
    await delay(100); // 每个填充操作后短暂延时，增加稳定性
}
// 填充题目内容的函数
async function fillQuestionContent(questionData) {
    console.log('开始填充题目内容');

    // 等待页面加载
    await delay(800);

    // 找到当前活动的题目表单容器
    let currentForm = document.querySelector('.question-item.active');
    if (!currentForm) {
        // 备用选择器：查找最后一个题目容器或当前编辑的题目
        const allQuestions = document.querySelectorAll('.question-item');
        if (allQuestions.length > 0) {
            currentForm = allQuestions[allQuestions.length - 1];
        }
    }
    if (!currentForm) {
        // 最后的备用选择器：查找包含编辑表单的容器
        currentForm = document.querySelector('.question-form') || 
                     document.querySelector('.question-content') ||
                     document.querySelector('.form-container') ||
                     document;
    }

    console.log('🎯 当前题目表单容器:', currentForm);
    console.log('📊 容器类名:', currentForm.className);
    
    // 调试：列出容器内所有可编辑元素
    const allEditableInContainer = currentForm.querySelectorAll('[contenteditable="true"]');
    console.log(`📋 容器内共找到 ${allEditableInContainer.length} 个可编辑元素`);

    // 步骤 3: 填充题干
    await fillEditableDiv(currentForm, '请录入题干', questionData.stem);

    // 等待内容保存
    await delay(300);

    // 步骤 4: 填充选项
    var optionInputs = currentForm.querySelectorAll('.options .ckeditor_div[contenteditable="true"]');
    if (optionInputs.length === 0) {
        // 备用选择器
        optionInputs = document.querySelectorAll('.options .ckeditor_div[contenteditable="true"]');
    }

    for (let i = 0; i < questionData.options.length; i++) {
        if (optionInputs[i]) {
            optionInputs[i].classList.remove('placeholder');
            optionInputs[i].innerHTML = questionData.options[i];
            triggerEvents(optionInputs[i]);
            console.log(`✅ 成功设置选项 ${String.fromCharCode(65 + i)}: ${questionData.options[i]}`);
        } else {
            console.warn(`⚠️ 找不到选项 ${String.fromCharCode(65 + i)} 的输入框`);
        }
        await delay(100); // 每个操作间短暂延时
    }

    // 步骤 5: 设置答案 (根据索引)
    var radioButtons = currentForm.querySelectorAll('.ant-radio-group input[type="radio"]');
    if (radioButtons.length === 0) {
        radioButtons = document.querySelectorAll('.ant-radio-group input[type="radio"]');
    }

    if (radioButtons[questionData.answer]) {
        radioButtons[questionData.answer].click();
        console.log(`✅ 成功设置答案: ${String.fromCharCode(65 + questionData.answer)}`);
    } else {
        console.warn(`⚠️ 找不到索引为 ${questionData.answer} 的答案单选按钮`);
    }
    await delay(100);

    // 步骤 6: 填充解析
    await fillEditableDiv(currentForm, '请录入解析', questionData.analysis);

    // 点击保存按钮
    var saveButton = document.querySelector('button.ant-btn.ant-btn-primary[data-v-4c71fb2d]');
    if (!saveButton) {
        // 备用选择器
        saveButton = document.querySelector('button.ant-btn.ant-btn-primary');
        if (!saveButton) {
            saveButton = Array.from(document.querySelectorAll('button')).find(btn =>
                btn.textContent.includes('保存') || btn.textContent.includes('保 存')
            );
        }
    }

    if (saveButton) {
        saveButton.click();
        console.log('✅ 已点击保存按钮');
        await delay(1000);
    } else {
        console.error('❌ 未找到保存按钮');
    }

    // 等待一下让内容保存
    await delay(500);
    console.log('题目内容填充完成');
}


/**
 * 触发一个元素上的多个事件，以模拟真实用户操作，确保框架能接收到变更
 * @param {HTMLElement} element - 目标元素
 */
function triggerEvents(element) {
    element.focus();
    // 触发一系列事件，确保兼容各种前端框架
    ['input', 'change', 'keyup', 'blur'].forEach(eventType => {
        element.dispatchEvent(new Event(eventType, { bubbles: true, cancelable: true }));
    });
}

/**
 * 模拟键盘输入到可编辑元素
 * @param {HTMLElement} element - 目标元素
 * @param {string} content - 要输入的内容（支持HTML）
 */
async function simulateContentInput(element, content) {
    if (!element) {
        console.warn('⚠️ 目标元素不存在，跳过填充');
        return;
    }

    element.focus();

    // 触发开始编辑事件
    element.dispatchEvent(new KeyboardEvent('keydown', { bubbles: true }));

    // 设置内容
    element.innerHTML = content;

    // 触发一系列输入相关事件
    const events = ['input', 'textInput', 'keyup', 'change'];
    events.forEach(eventType => {
        element.dispatchEvent(new Event(eventType, { bubbles: true, cancelable: true }));
    });

    // 触发结束编辑事件
    element.dispatchEvent(new Event('blur', { bubbles: true }));

    console.log("✅ 模拟键盘输入完成");

    // 短暂延时确保内容稳定
    await new Promise(resolve => setTimeout(resolve, 100));
}

/**
 * 触发元素事件，确保页面能识别到内容变化（优化版本）
 * @param {HTMLElement} element - 目标元素
 */
function triggerInputEvents(element) {
    if (!element) return;

    element.focus();
    ['input', 'change', 'keyup', 'blur'].forEach(eventType => {
        element.dispatchEvent(new Event(eventType, { bubbles: true, cancelable: true }));
    });
}

// 定位并点击最后一题的函数
async function locateAndClickLastQuestion() {
    // 查找所有题目容器
    var allQuestions = document.querySelectorAll('.question-item');

    if (allQuestions.length > 0) {
        // 获取最后一个题目
        var lastQuestion = allQuestions[allQuestions.length - 1];

        // 滚动到最后一题
        lastQuestion.scrollIntoView({ behavior: 'smooth', block: 'center' });

        // 点击最后一题
        lastQuestion.click();

        console.log('已点击最后一题，ID:', lastQuestion.id);

        // 等待一下让页面响应
        await new Promise(resolve => setTimeout(resolve, 500));

        return true;
    } else {
        console.log('未找到任何题目');
        return false;
    }
}

// 添加新题目的函数
async function addNewQuestion() {
    // 查找"添加题目"按钮 - 多种选择器
    var addButton = document.querySelectorAll('.add-operate-item')[1];

    if (!addButton) {
        // 备用选择器1：通过文本内容查找
        addButton = Array.from(document.querySelectorAll('button, .add-operate-item')).find(btn =>
            btn.textContent && btn.textContent.includes('添加题目')
        );
    }

    if (!addButton) {
        // 备用选择器2：通过类名查找
        addButton = document.querySelector('.add-operate-item');
    }

    if (addButton) {
        // 点击添加题目按钮
        addButton.click();
        console.log('✅ 已点击添加题目按钮');

        // 等待新题目创建完成
        await delay(1000); // 增加等待时间，确保题目完全创建
        return true;
    } else {
        console.warn('⚠️ 未找到添加题目按钮，可能已在编辑状态');
        return false;
    }
}

// 主执行函数
async function main() {
    try {
        console.log(`🚀 脚本启动，准备处理 ${Questions.length} 道单选题...`);

        for (let i = 0; i < Questions.length; i++) {
            const questionData = Questions[i];
            console.log(`\n--- [ ${i + 1} / ${Questions.length} ] --- 开始处理第 ${i + 1} 个题目`);

            // 1. 先定位并点击最后一题
            const locateSuccess = await locateAndClickLastQuestion();
            if (!locateSuccess) {
                console.error(`第 ${i + 1} 个题目：无法定位到最后一题`);
                continue;
            }

            // 2. 添加新题目（如果不是第一题）
            const addSuccess = await addNewQuestion();
            if (!addSuccess) {
                console.error(`第 ${i + 1} 个题目：无法添加新题目`);
                continue;
            }

            // 3. 再次定位到新创建的最后一题
            await locateAndClickLastQuestion();


            // 4. 设置题型为单选题
            const typeSetSuccess = await operateElements();
            if (!typeSetSuccess) {
                console.warn(`第 ${i + 1} 个题目：题型设置可能失败，继续尝试填充内容`);
            }





            // 获取所有选项关闭按钮（X）并删除第一个
            const optionCloseButtons = document.querySelectorAll('.options-close');
            if (optionCloseButtons.length > 0) {
                optionCloseButtons[0].click();
                console.log('✅ 已点击第一个选项关闭按钮');
                await delay(300);
            } else {
                console.warn('⚠️ 未找到选项关闭按钮');
            }


            // 5. 填充题目内容
            await fillQuestionContent(questionData);

            console.log(`✅ 第 ${i + 1} 个题目处理完成`);

            // 每个题目之间稍作停顿
            await delay(500);
        }

        console.log('\n🎉🎉🎉 所有题目处理完成！');
    } catch (error) {
        console.error('💥 执行过程中出现错误:', error);
        console.error('请检查页面结构或刷新页面后重试。');
    }
}

// 执行主函数
main(); 
"#,
        )
    }

    fn get_muti_tiankong_code(&self) -> String {
        String::from(
            r#"
/**
 * 等待指定毫秒数
 * @param {number} ms - 等待的时间（毫秒）
 */
var delay = (ms) => new Promise(resolve => setTimeout(resolve, ms));

//MARK： 使用XPath查找包含指定文本的元素
function clickBlankFillingElement(type) {
    // XPath表达式：查找class包含"tag"且包含指定文本的元素
    var xpath = "//*[contains(@class,'tag') and contains(text(),'" + type + "')]";

    // 执行XPath查询
    var result = document.evaluate(
        xpath,
        document,
        null,
        XPathResult.FIRST_ORDERED_NODE_TYPE,
        null
    );

    // 如果找到元素，点击它
    if (result.singleNodeValue) {
        result.singleNodeValue.click();
        console.log(`成功点击 ${type} 标签元素`);
        return true;
    } else {
        console.log(`未找到包含 '${type}' 文本的标签元素`);
        return false;
    }
}

// 完整的操作流程
async function operateElements(type) {
    console.log(`开始设置题型: ${type}`);

    // 1. 点击下拉框
    var selectDiv = document.querySelector('div[title="单选题"]');

    if (selectDiv) {
        selectDiv.click();
        console.log('已点击题型下拉框');

        // 2. 选择指定题型 - 使用 Promise 替代 setTimeout
        await new Promise(resolve => {
            setTimeout(function () {
                var options = document.querySelectorAll('li.ant-select-dropdown-menu-item');
                var found = false;

                for (var i = 0; i < options.length; i++) {
                    if (options[i].textContent.trim() === '填空题') {
                        options[i].click();
                        // console.log(`已选择题型: ${type}`);
                        found = true;
                        break;
                    }
                }

                if (!found) {
                    console.warn(`未找到题型选项: 填空题`);
                }
                resolve();
            }, 200);
        });

        // 3. 使用XPath点击对应标签
        await new Promise(resolve => {
            setTimeout(function () {
                const success = clickBlankFillingElement('单词拼写');
                if (success) {
                    console.log(`已点击单词拼写标签`);
                } else {
                    console.warn(`未能点击单词拼写标签`);
                }
                resolve();
            }, 300);
        });
    } else {
        console.error('未找到题型下拉框');
    }
}

// 填充题目内容的函数
async function fillQuestionContent(questionData) {
    console.log('开始填充题目内容');

    // 等待页面加载
    await new Promise(resolve => setTimeout(resolve, 800));

    // 填充题干内容 - 针对填空题的编辑器
    var stemEditor = document.querySelector('.ql-editor[data-placeholder="请录入题干"]');
    if (!stemEditor) {
        // 备用选择器 - 查找题干内容编辑器
        stemEditor = document.querySelector('div[contenteditable="true"][placeholder*="题干"]');
        if (!stemEditor) {
            // 再次备用 - 查找第一个可编辑的内容区域
            stemEditor = document.querySelector('.ql-editor');
        }
    }

    if (stemEditor) {
        await simulateContentInput(stemEditor, questionData.stem);
        console.log('✅ 已填充题干内容');
    } else {
        console.error('❌ 未找到题干编辑器');
    }

    // 等待内容保存
    await new Promise(resolve => setTimeout(resolve, 300));

    // 填充答案内容 - 使用优化的fillBlankAnswers方法
    if (questionData.answer && questionData.answer.length > 0) {
        await fillBlankAnswers(questionData.answer);
    }

    // 等待答案保存
    await new Promise(resolve => setTimeout(resolve, 300));

    // 填充解析内容
    var analysisEditor = document.querySelector('.ql-editor[data-placeholder="请输入解析"]');
    if (!analysisEditor) {
        // 备用选择器 - 查找解析编辑器
        analysisEditor = document.querySelector('div[contenteditable="true"][placeholder*="解析"]');
        if (!analysisEditor) {
            // 查找所有编辑器，取第二个（通常是解析）
            const allEditors = document.querySelectorAll('.ql-editor');
            if (allEditors.length > 1) {
                analysisEditor = allEditors[1];
            }
        }
    }

    if (analysisEditor) {
        await simulateContentInput(analysisEditor, questionData.analysis);
        console.log('✅ 已填充解析内容');
    } else {
        console.error('❌ 未找到解析编辑器');
    }
    // 点击保存按钮
    var saveButton = document.querySelector('button.ant-btn.ant-btn-primary[data-v-4c71fb2d]');
    if (saveButton && saveButton.textContent.includes('保 存')) {
        saveButton.click();
        console.log('✅ 已点击保存按钮');
        await new Promise(resolve => setTimeout(resolve, 1000));
    } else {
        console.error('❌ 未找到保存按钮');
    }
    // 等待一下让内容保存
    await new Promise(resolve => setTimeout(resolve, 500));
    console.log('题目内容填充完成');
}

/**
 * 模拟键盘输入到可编辑元素
 * @param {HTMLElement} element - 目标元素
 * @param {string} content - 要输入的内容（支持HTML）
 */
async function simulateContentInput(element, content) {
    if (!element) {
        console.warn('⚠️ 目标元素不存在，跳过填充');
        return;
    }

    element.focus();

    // 触发开始编辑事件
    element.dispatchEvent(new KeyboardEvent('keydown', { bubbles: true }));

    // 设置内容
    element.innerHTML = content;

    // 触发一系列输入相关事件
    const events = ['input', 'textInput', 'keyup', 'change'];
    events.forEach(eventType => {
        element.dispatchEvent(new Event(eventType, { bubbles: true, cancelable: true }));
    });

    // 触发结束编辑事件
    element.dispatchEvent(new Event('blur', { bubbles: true }));

    console.log("✅ 模拟键盘输入完成");

    // 短暂延时确保内容稳定
    await new Promise(resolve => setTimeout(resolve, 100));
}

/**
 * 优化的填空题答案填充函数（基于fillBlankAnswers方法）
 * @param {Array} blankAnswers - 答案数组
 */
async function fillBlankAnswers(blankAnswers) {
    console.log(`🚀 开始填充 ${blankAnswers.length} 个填空题答案...`);

    try {
        // 找到所有的填空输入框
        const blankInputs = document.querySelectorAll('.blanks-value .ckeditor_div[contenteditable="true"][placeholder="请录入答案"]');
        
        console.log(`📝 找到 ${blankInputs.length} 个填空输入框`);

        if (blankInputs.length === 0) {
            console.warn("⚠️ 未找到专用填空输入框，尝试备用方法...");
            
            // 备用方法1：查找原有的答案编辑器
            let answerEditor = document.querySelector('.ckeditor_div.whiteOnly.showBox.placeholderText');
            if (!answerEditor) {
                // 备用方法2：通过样式查找
                answerEditor = document.querySelector('div[style*="background: rgb(242, 242, 242)"]');
                if (!answerEditor) {
                    // 备用方法3：通过包含"请录入答案"文本查找
                    const allDivs = document.querySelectorAll('div');
                    for (let div of allDivs) {
                        if (div.textContent.includes('请录入答案')) {
                            answerEditor = div;
                            break;
                        }
                    }
                }
            }

            if (answerEditor) {
                // 对于单个答案编辑器，将所有答案用换行分隔
                const answerText = blankAnswers.filter(answer => answer.trim() !== '').join('\n');
                
                // 清空原有内容并设置焦点
                answerEditor.focus();
                answerEditor.innerHTML = '';
                answerEditor.textContent = '';

                // 模拟键盘输入答案内容
                await simulateTypingInput(answerEditor, answerText);
                // 触发事件确保页面识别到变化
                triggerInputEvents(answerEditor);

                console.log('✅ 已通过备用方法填充答案内容:', answerText);
                return;
            }

            // 最后的备用方法：查找其他输入框
            const inputElements = document.querySelectorAll('input[type="text"], textarea, div[contenteditable="true"]');
            for (let element of inputElements) {
                const parentText = element.parentElement?.textContent || '';
                if (parentText.includes('答案') || parentText.includes('Answer')) {
                    console.log('找到其他答案输入框');
                    const answerText = blankAnswers.filter(answer => answer.trim() !== '').join(', ');
                    if (element.tagName === 'INPUT' || element.tagName === 'TEXTAREA') {
                        element.value = '';
                        element.focus();
                        await simulateTypingInput(element, answerText);
                    } else {
                        await simulateTypingInput(element, answerText);
                    }
                    console.log('✅ 已通过最终备用方法填充答案:', answerText);
                    return;
                }
            }

            console.error("❌ 未找到任何可用的答案输入框！");
            return;
        }

        // 主要方法：逐个填充每个填空输入框
        for (let i = 0; i < Math.min(blankAnswers.length, blankInputs.length); i++) {
            const inputElement = blankInputs[i];
            const answer = blankAnswers[i];

            if (inputElement && answer && answer.trim() !== '') {
                // 移除占位符样式
                inputElement.classList.remove('placeholder');
                
                // 填入答案内容
                inputElement.innerHTML = answer;
                
                // 触发事件确保页面识别到变化
                triggerInputEvents(inputElement);
                
                console.log(`✅ 空${i + 1} 填充完成: ${answer}`);
                
                // 每个填充操作后短暂延时
                await delay(200);
            } else {
                console.warn(`⚠️ 空${i + 1} 填充失败: ${!inputElement ? '找不到输入框' : '答案为空'}`);
            }
        }

        console.log("\n🎉 所有填空题答案填充完成！");

    } catch (error) {
        console.error("💥 填充答案过程中发生错误:", error);
    }
}

/**
 * 触发元素事件，确保页面能识别到内容变化（优化版本）
 * @param {HTMLElement} element - 目标元素
 */
function triggerInputEvents(element) {
    if (!element) return;
    
    element.focus();
    ['input', 'change', 'keyup', 'blur'].forEach(eventType => {
        element.dispatchEvent(new Event(eventType, { bubbles: true, cancelable: true }));
    });
}

/**
 * 模拟逐字符键盘输入，更真实地模拟用户打字
 * @param {HTMLElement} element - 目标元素
 * @param {string} text - 要输入的文本
 */
async function simulateTypingInput(element, text) {
    if (!element || !text) {
        console.warn('⚠️ 目标元素或文本不存在，跳过模拟键盘输入');
        return;
    }

    element.focus();

    // 触发开始输入事件
    element.dispatchEvent(new Event('focus', { bubbles: true }));
    element.dispatchEvent(new KeyboardEvent('keydown', { bubbles: true, key: 'Process' }));

    // 逐字符输入
    for (let i = 0; i < text.length; i++) {
        const char = text[i];

        // 模拟按键事件
        element.dispatchEvent(new KeyboardEvent('keydown', {
            bubbles: true,
            key: char,
            code: `Key${char.toUpperCase()}`,
            keyCode: char.charCodeAt(0)
        }));

        // 添加字符到内容
        if (element.tagName === 'INPUT' || element.tagName === 'TEXTAREA') {
            element.value = text.substring(0, i + 1);
        } else {
            element.textContent = text.substring(0, i + 1);
        }

        // 触发输入事件
        element.dispatchEvent(new Event('input', { bubbles: true, cancelable: true }));
        element.dispatchEvent(new InputEvent('input', {
            bubbles: true,
            cancelable: true,
            data: char,
            inputType: 'insertText'
        }));

        // 模拟按键释放
        element.dispatchEvent(new KeyboardEvent('keyup', {
            bubbles: true,
            key: char,
            code: `Key${char.toUpperCase()}`,
            keyCode: char.charCodeAt(0)
        }));

        // 短暂延时模拟真实打字速度
        await new Promise(resolve => setTimeout(resolve, 30 + Math.random() * 50));
    }

    // 触发结束输入事件
    element.dispatchEvent(new Event('change', { bubbles: true, cancelable: true }));
    element.dispatchEvent(new KeyboardEvent('keyup', { bubbles: true }));
    element.dispatchEvent(new Event('blur', { bubbles: true }));

    console.log("✅ 模拟键盘输入完成:", text);

    // 额外延时确保内容稳定
    await new Promise(resolve => setTimeout(resolve, 200));
}

// 定位并点击最后一题的函数
async function locateAndClickLastQuestion() {
    // 查找所有题目容器
    var allQuestions = document.querySelectorAll('.question-item');

    if (allQuestions.length > 0) {
        // 获取最后一个题目
        var lastQuestion = allQuestions[allQuestions.length - 1];

        // 滚动到最后一题
        lastQuestion.scrollIntoView({ behavior: 'smooth', block: 'center' });

        // 点击最后一题
        lastQuestion.click();

        console.log('已点击最后一题，ID:', lastQuestion.id);

        // 等待一下让页面响应
        await new Promise(resolve => setTimeout(resolve, 500));

        return true;
    } else {
        console.log('未找到任何题目');
        return false;
    }
}

// 添加新题目的函数
async function addNewQuestion() {
    // 查找"添加题目"按钮
    var addButton = document.querySelectorAll('.add-operate-item')[1];

    if (addButton) {
        // 点击添加题目按钮
        addButton.click();
        console.log('已点击添加题目按钮');

        // 等待新题目创建完成
        await new Promise(resolve => setTimeout(resolve, 1000));
        return true;
    } else {
        console.log('未找到添加题目按钮');
        return false;
    }
}

// 主执行函数
async function main() {
    try {
        for (let i = 0; i < Questions.length; i++) {
            const timu = Questions[i];
            console.log(`开始处理第 ${i + 1} 个题目: ${timu.题型类型}`);

            // 1. 先定位并点击最后一题
            const locateSuccess = await locateAndClickLastQuestion();
            if (!locateSuccess) {
                console.error(`第 ${i + 1} 个题目：无法定位到最后一题`);
                continue;
            }

            // 2. 添加新题目
            const addSuccess = await addNewQuestion();
            if (!addSuccess) {
                console.error(`第 ${i + 1} 个题目：无法添加新题目`);
                continue;
            }

            // 3. 再次定位到新创建的最后一题
            await locateAndClickLastQuestion();

            // 4. 设置题型
            await operateElements(timu.题型类型);

            // 5. 填充题目内容
            await fillQuestionContent(timu);

            console.log(`第 ${i + 1} 个题目处理完成`);

            // 每个题目之间稍作停顿
            await new Promise(resolve => setTimeout(resolve, 800));
        }
        console.log('所有题目处理完成！');
    } catch (error) {
        console.error('执行过程中出现错误:', error);
    }
}

// 执行主函数
main();   
"#,
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
