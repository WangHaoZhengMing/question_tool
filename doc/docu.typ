#import "@preview/grape-suite:3.1.0": exercise
#import exercise: project, task, subtask

#show: project.with(
    title: "Lorem ipsum dolor sit",

    university: [University],
    institute: [Institute],
    seminar: [Seminar],

    abstract: lorem(100),
    show-outline: true,

    author: "John Doe",

    show-solutions: false
)
#set text(font: "simhei")
#set page(margin: (top: 2cm, bottom: 2cm, left: 2cm, right: 2cm))

// 封面页
#page(
  margin: (top: 3cm, bottom: 3cm, left: 3cm, right: 3cm),
  background: rect(
    width: 100%,
    height: 100%,
    fill: gradient.linear(
      rgb("#1e3a8a"), 
      rgb("#3b82f6"), 
      rgb("#60a5fa"),
      angle: 45deg
    )
  )
)[
  #v(1fr)
  
  // 主标题区域
  #align(center)[
    #rect(
      width: 85%,
      radius: 15pt,
      fill: rgb("#ffffff").transparentize(5%),
      stroke: none,
      inset: (x: 30pt, y: 40pt)
    )[
      #align(center)[
        // 图标/徽标区域
        #rect(
          width: 80pt,
          height: 80pt,
          radius: 40pt,
          fill: gradient.radial(
            rgb("#1e40af"),
            rgb("#3b82f6"),
          ),
          stroke: (thickness: 3pt, paint: white)
        )[
          #align(center + horizon)[
            #text(
              size: 36pt,
              fill: white,
              weight: "bold"
            )[Q]
          ]
        ]
        
        #v(25pt)
        
        // 主标题
        #text(
          size: 32pt,
          weight: "bold",
          fill: rgb("#1e40af")
        )[Question Tool]
        
        #v(10pt)
        
        // 副标题
        #text(
          size: 18pt,
          fill: rgb("#374151"),
          style: "italic"
        )[AI驱动的题目录入自动化工具]
        
        #v(20pt)
        
        // 分割线
        #line(
          length: 70%,
          stroke: (
            thickness: 2pt,
            paint: gradient.linear(
              rgb("#3b82f6"),
              rgb("#60a5fa"),
              rgb("#93c5fd")
            )
          )
        )
        
        #v(20pt)
        
        // 版本信息
        #rect(
          radius: 8pt,
          fill: rgb("#eff6ff"),
          stroke: (thickness: 1pt, paint: rgb("#3b82f6")),
          inset: 12pt
        )[
          #text(
            size: 14pt,
            weight: "medium",
            fill: rgb("#1e40af")
          )[技术文档 v1.0]
        ]
        
        #v(30pt)
        
        // 核心特性标签
        #grid(
          columns: 2,
          gutter: 15pt,
          
          rect(
            radius: 20pt,
            fill: rgb("#dcfce7"),
            stroke: (thickness: 1pt, paint: rgb("#16a34a")),
            inset: (x: 15pt, y: 8pt)
          )[
            #align(center)[
              #text(
                size: 10pt,
                weight: "bold",
                fill: rgb("#166534")
              )[🚀 效率提升 20×]
            ]
          ],
          
          rect(
            radius: 20pt,
            fill: rgb("#fef3c7"),
            stroke: (thickness: 1pt, paint: rgb("#d97706")),
            inset: (x: 15pt, y: 8pt)
          )[
            #align(center)[
              #text(
                size: 10pt,
                weight: "bold",
                fill: rgb("#92400e")
              )[🤖 AI 智能生成]
            ]
          ]
        )
      ]
    ]
  ]
  
  #v(1fr)
  
  // 底部信息
  #align(center)[
    #rect(
      width: 70%,
      radius: 10pt,
      fill: rgb("#ffffff").transparentize(10%),
      stroke: none,
      inset: 20pt
    )[
      #align(center)[
        #text(
          size: 12pt,
          fill: rgb("#6b7280"),
          weight: "medium"
        )[河南工业大学 · 计算机科学与技术]
        
        #v(5pt)
        
        #text(
          size: 11pt,
          fill: rgb("#9ca3af"),
          style: "italic"
        )[王浩然 · 2025年10月]
        
        #v(8pt)
        
        #text(
          size: 10pt,
          fill: rgb("#9ca3af")
        )[郑州新东方 · 录排实习项目]
      ]
    ]
  ]
  
  #v(0.5fr)
]

#pagebreak()

= 项目概述

== 产品简介
#image("img/homepage.png")
Question Tool 是一个基于 Rust 和 Slint UI 框架开发的智能题目录入工具，专为教育机构和内容创作者设计。通过集成多种大语言模型后端（OpenAI GPT、GitHub Models 等），实现从剪贴板内容自动生成标准化题目，并通过 JavaScript 自动化脚本完成网页表单的批量录入。

== 核心价值主张

#grid(
  columns: (1fr, 1fr),
  gutter: 20pt,
  [
    === 效率革命
    - 手动录入效率提升 #text(weight: "bold", fill: red)[20倍]
    - 每周节省 #text(weight: "bold")[10+ 小时] 工作时间
  ],
  [
    === 智能化生成
    - AI 驱动的内容生成，确保专业质量
    - 六种题型全覆盖，标准化输出
    - 从内容识别到网页填充的完整自动化流程
  ]
)

= 功能特性

== 支持的题目类型

Question Tool 目前支持以下六种标准化题目类型：

#grid(
  columns: (1fr, 1fr),
  gutter: 15pt,
  [
    #rect(
      fill: rgb("#f0f8ff"),
      stroke: rgb("#4682b4"),
      width: 100%,
      radius: 5pt,
      inset: 10pt,
    )[
      === 单选题
      #text(style: "italic", size: 10pt)[Single Choice]
      
      + 自动生成选项 A、B、C、D
      + 智能答案标记和解析  
      + 符合标准化考试格式
    ]
    
    #v(10pt)
    
    #rect(
      fill: rgb("#f0fff0"),
      stroke: rgb("#32cd32"),
      width: 100%,
      radius: 5pt,
      inset: 10pt,
    )[
      === 完形填空
      #text(style: "italic", size: 10pt)[Cloze Test]
      
      + 文章挖空处理
      + 选项匹配和语法分析
      + 自动生成标注
    ]
    
    #v(10pt)
    
    #rect(
      fill: rgb("#fff8dc"),
      stroke: rgb("#daa520"),
      width: 100%,
      radius: 5pt,
      inset: 10pt,
    )[
      === 单项听力理解  
      #text(style: "italic", size: 10pt)[Listening Single]
      
      + 音频材料描述生成
      + 口语化表达识别
      + 情景对话分析
    ]
  ],
  [
    #rect(
      fill: rgb("#fff0f5"),
      stroke: rgb("#db7093"),
      width: 100%,
      radius: 5pt,
      inset: 10pt,
    )[
      === 阅读理解
      #text(style: "italic", size: 10pt)[Reading Comprehension]
      
      + 文章段落格式化处理
      + 多题目组合生成
      + 考点分析和答题技巧
    ]
    
    #v(10pt)
    
    #rect(
      fill: rgb("#f5f5dc"),
      stroke: rgb("#cd853f"),
      width: 100%,
      radius: 5pt,
      inset: 10pt,
    )[
      === 听力复合题
      #text(style: "italic", size: 10pt)[Listening Compound]
      
      + 长对话材料处理
      + 智能答案标记和解析  
      + 复杂逻辑关系分析
    ]
    
    #v(10pt)
    
    #rect(
      fill: rgb("#f0f0f0"),
      stroke: rgb("#696969"),
      width: 100%,
      radius: 5pt,
      inset: 10pt,
    )[
      === 多项填空
      #text(style: "italic", size: 10pt)[Multi Blank Filling]
      
      + 知识点填空
      + 概念定义匹配
      + 智能答案标记和解析  
    ]
  ]
)

以及听力音频的生成和处理。（待整合，代码己完成）
用户只需将文本复制到剪贴板，Question Tool 即可自动识别对话类型并自动生成标准的 Toml 文件，进而生成*全部*音频文件。
#image("img/rust_tts_code_overview.png")

#image("img/split_audio_code_overview.png")

== 技术架构

#align(center)[
  #rect(
    fill: rgb("#f8f9fa"),
    stroke: rgb("#6c757d"),
    width: 80%,
    radius: 5pt,
    inset: 15pt,
  )[
    #align(center)[
      #text(16pt, weight: "bold")[Question Tool 系统架构]
      #linebreak()
      #text(12pt, style: "italic")[前端 ↔️ 后端 ↔️ 自动化脚本]
    ]
  ]
]

#grid(
  columns: 3,
  gutter: 15pt,
  [
    #align(center)[
      #rect(
        fill: rgb("#e6f3ff"),
        stroke: rgb("#0066cc"),
        width: 100%,
        radius: 8pt,
        inset: 12pt,
      )[
        #align(center)[
          #text(14pt, weight: "bold", fill: rgb("#0066cc"))[前端界面]
          #line(length: 100%, stroke: rgb("#0066cc"))
        ]
        
        #text(weight: "bold")[框架]: Slint UI
        #linebreak()
        #text(size: 9pt, style: "italic")[Rust 原生 UI 框架]
        
        #v(8pt)
        
        #text(weight: "bold")[特性]:
        + 现代化界面设计
        + 流畅的用户体验
        + 实时剪贴板监控
        + 内容预览功能
      ]
    ]
  ],
  [
    #align(center)[
      #rect(
        fill: rgb("#fff2e6"),
        stroke: rgb("#ff6600"),
        width: 100%,
        radius: 8pt,
        inset: 12pt,
      )[
        #align(center)[
          #text(14pt, weight: "bold", fill: rgb("#ff6600"))[后端处理]
          #line(length: 100%, stroke: rgb("#ff6600"))
        ]
        
        #text(weight: "bold")[语言]: Rust
        #linebreak()
        #text(size: 9pt, style: "italic")[高性能、内存安全]
        
        #v(8pt)
        
        #text(weight: "bold")[组件]:
        + Tokio 异步运行时
        + 多 LLM 后端支持
        + 剪贴板监控服务
        + 内容处理引擎
      ]
    ]
  ],
  [
    #align(center)[
      #rect(
        fill: rgb("#f0ffe6"),
        stroke: rgb("#66cc00"),
        width: 100%,
        radius: 8pt,
        inset: 12pt,
      )[
        #align(center)[
          #text(14pt, weight: "bold", fill: rgb("#66cc00"))[自动化脚本]
          #line(length: 100%, stroke: rgb("#66cc00"))
        ]
        
        #text(weight: "bold")[语言]: JavaScript
        #linebreak()
        #text(size: 9pt, style: "italic")[浏览器执行环境]
        
        #v(8pt)
        
        #text(weight: "bold")[功能]:
        + DOM 智能操作
        + 表单自动填充
        + 用户事件模拟
        + 跨浏览器兼容
      ]
    ]
  ]
)

#pagebreak()

= 性能数据与优化

== 效率对比分析

#figure(caption: [对比])[#table(
  columns: 4,
  align: center,
  stroke: rgb("#dee2e6"),
  fill: (x, y) => if y == 0 { rgb("#e9ecef") } else if calc.odd(y) { rgb("#f8f9fa") } else { white },
  
  [#text(weight: "bold")[题目类型]], 
  [#text(weight: "bold")[手动录入时间]], 
  [#text(weight: "bold")[自动化处理时间]], 
  [#text(weight: "bold")[效率提升倍数]],
  
  [完形填空], [20分钟], [50秒], [#text(fill: red, weight: "bold")[24×]],
  [阅读理解], [15分钟], [40秒], [#text(fill: red, weight: "bold")[22.5×]],
  [单选题], [8分钟], [35秒], [#text(fill: red, weight: "bold")[13.7×]],
  [听力题], [12分钟], [45秒], [#text(fill: red, weight: "bold")[16×]],
)]

#v(20pt)

== 系统性能优化

#grid(
  columns: (1fr, 1fr),
  gutter: 20pt,
  [
    === 内存管理优化
    
    #rect(
      fill: rgb("#d4edda"),
      stroke: rgb("#c3e6cb"),
      width: 100%,
      radius: 5pt,
      inset: 10pt,
    )[
      #grid(
        columns: (auto, 1fr),
        gutter: 10pt,
        [优化前:], [349MB],
        [优化后:], [~80MB],
        [优化幅度:], [#text(fill: green, weight: "bold")[77% ↓]],
      )
    ]
  ],
  [
    === 运行时性能
    
    #table(
      columns: 2,
      align: (left, center),
      stroke: rgb("#dee2e6"),
      fill: (x, y) => if calc.odd(y) { rgb("#f8f9fa") } else { white },
      
      [启动时间], [< 2秒],
      [响应延迟], [< 100ms],
      [稳定性], [7×24小时],
      [内存泄漏], [零检出],
    )
    
  ]
)

= 工作流程

#align(center)[
  #rect(
    fill: rgb("#f8f9fa"),
    stroke: rgb("#6c757d"),
    width: 90%,
    radius: 8pt,
    inset: 20pt,
  )[
    #text(16pt, weight: "bold")[智能化题目处理流程]
    
    #v(15pt)
    
    #grid(
      columns: 4,
      gutter: 10pt,
      
      // 步骤1
      align(center)[
        #circle(
          fill: rgb("#007bff"),
          radius: 15pt,
        )[
          #text(fill: white, weight: "bold", size: 14pt)[1]
        ]
        #v(5pt)
        #text(10pt, weight: "bold")[内容识别]
      ],
      
      // 箭头
      align(center)[
        #text(20pt)[→]
      ],
      
      // 步骤2
      align(center)[
        #circle(
          fill: rgb("#28a745"),
          radius: 15pt,
        )[
          #text(fill: white, weight: "bold", size: 14pt)[2]
        ]
        #v(5pt)
        #text(10pt, weight: "bold")[AI 生成]
      ],
      
      // 箭头
      align(center)[
        #text(20pt)[→]
      ],
    )
    
    #v(10pt)
    
    #grid(
      columns: 4,
      gutter: 10pt,
      
      // 步骤4 (反向)
      align(center)[
        #circle(
          fill: rgb("#6f42c1"),
          radius: 15pt,
        )[
          #text(fill: white, weight: "bold", size: 14pt)[4]
        ]
        #v(5pt)
        #text(10pt, weight: "bold")[浏览器执行]
      ],
      
      // 箭头
      align(center)[
        #text(20pt)[←]
      ],
      
      // 步骤3
      align(center)[
        #circle(
          fill: rgb("#fd7e14"),
          radius: 15pt,
        )[
          #text(fill: white, weight: "bold", size: 14pt)[3]
        ]
        #v(5pt)
        #text(10pt, weight: "bold")[自动填充]
      ],
      
      // 空白
      [],
    )
  ]
]

#v(20pt)

== 详细流程说明

#grid(
  columns: (1fr, 1fr),
  gutter: 15pt,
  [
    === 1️⃣ 内容识别
    #rect(
      fill: rgb("#e3f2fd"),
      stroke: rgb("#2196f3"),
      width: 100%,
      radius: 5pt,
      inset: 10pt,
    )[
      + 实时监控剪贴板内容
      + 自动识别图片中的文本 (OCR)
      + 智能分析内容类型和结构
      + 预处理和格式规范化
    ]
    
    #v(10pt)
    
    === 3️⃣ 自动填充
    #rect(
      fill: rgb("#fff3e0"),
      stroke: rgb("#ff9800"),
      width: 100%,
      radius: 5pt,
      inset: 10pt,
    )[
      + 生成对应的 JavaScript 自动化代码
      + 模拟用户操作填充网页表单
      + 处理各种表单控件和验证逻辑
      + 批量处理和错误重试机制
    ]
  ],
  [
    === 2️⃣ AI 生成
    #rect(
      fill: rgb("#e8f5e8"),
      stroke: rgb("#4caf50"),
      width: 100%,
      radius: 5pt,
      inset: 10pt,
    )[
      + 根据内容类型选择合适的提示模板
      + 调用 LLM API 生成标准化题目
      + 确保输出格式符合平台要求
      + 多轮优化和内容校验
    ]
    
    #v(10pt)

    === 4️⃣ 浏览器执行
    #rect(
      fill: rgb("#f3e5f5"),
      stroke: rgb("#9c27b0"),
      width: 100%,
      radius: 5pt,
      inset: 10pt,
    )[
      + 在浏览器环境中执行自动化脚本
      + 兼容主流浏览器 (Chrome、Firefox、Edge)
      + 处理动态加载和异步内容
      + 提供执行日志和结果反馈
    ]
  ]
)

== 开发背景

=== 项目起源

本项目由河南工业大学计算机科学与技术专业大三学生王浩然在郑州新东方录排实习期间开发。针对教育内容制作中的实际痛点，通过技术创新大幅提升工作效率。

=== 解决的问题

*传统工作流的挑战:*
- 手动录入题目耗时长，容易出错
- 格式标准化要求严格，重复性工作量大
- 不同题型的处理逻辑复杂，难以标准化

*技术解决方案:*
- AI 驱动的内容生成，确保质量和一致性
- 自动化脚本处理重复性操作
- 模块化架构支持快速扩展新题型
- 内存优化确保长时间稳定运行

== 技术创新

=== 1. 多后端 LLM 支持
```rust
pub enum LLMBackend {
    OpenAI,
    GitHub,
    // 易于扩展新的 AI 服务
}
```

=== 2. 智能提示模板系统
每种题型都有专门优化的提示模板，确保生成内容的专业性和准确性。

=== 3. 内存管理优化
- 临时文件自动清理机制
- 图片对象智能释放
- 全局 Tokio Runtime 复用

=== 4. 跨平台兼容性
基于 Rust 和 Slint 的技术栈确保在 Windows、macOS 和 Linux 上的一致体验。

== 未来规划

=== 短期优化 (v0.2.0)
- *性能提升*: 进一步优化代码执行速度
- *UI 改进*: 增强用户界面的交互体验
- *错误处理*: 完善异常情况的处理和恢复机制

=== 中期发展 (v1.0.0)
- *API 集成*: 支持直接访问题库数据库
- *题型扩展*: 增加更多专业领域的题目类型
- *多个学科支持*: 扩展到数学、物理等多个学科领域
- *并发处理*: 支持多任务并发执行，提高处理效率

=== 长期愿景
- *云端服务*: 提供 SaaS 版本，支持团队协作
- *智能推荐*: 基于使用数据的个性化优化建议

== 技术规格

=== 系统要求
- *操作系统*: Windows 7+ / macOS 10.15+ / Ubuntu 18.04+
- *内存*: 最低 4GB，推荐 8GB
- *存储*: 至少 100MB 可用空间
- *网络*: 稳定的互联网连接 (用于 AI API 调用)

=== 开发环境
- *Rust*: 1.75+
- *Node.js*: 16+ (用于构建脚本)
- *Git*: 版本控制和协作

== 实际应用案例

=== 案例一：完形填空题目处理
*场景描述：*
处理一套标准英语完形填空题，包含20个空格，需要生成选项和标准答案。

*处理流程：*
1. 从剪贴板获取原始文章内容
2. AI 识别需要挖空的关键词位置
3. 识别选项
4. 自动生成考点分析和解题思路
5. 一键填充到网页表单中

*效率对比：*
- 传统方式：20分钟（包括选项设计、格式调整、手动录入）
- 自动化处理：50秒（AI生成+自动填充）
- 效率提升：24倍

=== 案例二：阅读理解题组
*场景描述：*
处理包含5道题目的阅读理解材料，涉及细节理解、推理判断等多种题型。

*处理过程：*
1. 从剪贴板获取原始文章内容
2. 创建题目
3. 每道题目配备详细的解析说明
4. 自动设置答案

*效率提升：*
- 传统耗时：15分钟
- 自动化耗时：50秒
- 时间节省：20倍效率提升

#pagebreak()

= 总结与展望

== 项目价值与意义

#rect(
  fill: rgb("#f8f9fa"),
  stroke: rgb("#6c757d"),
  width: 100%,
  radius: 8pt,
  inset: 15pt,
)[
  Question Tool 代表了 #text(weight: "bold", fill: blue)[教育技术领域 AI 应用] 的一个成功实践案例。通过将前沿的语言模型技术与实际的教育内容制作需求相结合，成功创造了一个能够显著提升工作效率的实用工具。
]

#v(15pt)

#grid(
  columns: (1fr, 1fr),
  gutter: 20pt,
  [
    === 技术层面的突破
    
    + #text(weight: "bold")[性能优化]: 24倍效率提升，内存占用降低77%
    + #text(weight: "bold")[架构设计]: 模块化、可扩展的系统架构
    + #text(weight: "bold")[技术创新]: Rust + AI + 自动化的完美结合
    + #text(weight: "bold")[跨平台兼容]: 支持主流操作系统和浏览器
  ],
  [
    === 社会价值与影响
    
    + #text(weight: "bold")[效率革命]: 为教育工作者每周节省10+小时
    + #text(weight: "bold")[质量提升]: AI驱动确保内容专业性和一致性
    + #text(weight: "bold")[成本降低]: 大幅减少人力投入和运营成本
    + #text(weight: "bold")[创新示范]: 为教育技术发展提供参考案例
  ]
)

#v(20pt)

== 致谢

#align(center)[
  #rect(
    fill: rgb("#e8f5e8"),
    stroke: rgb("#4caf50"),
    width: 80%,
    radius: 8pt,
    inset: 15pt,
  )[
    感谢 #text(weight: "bold")[郑州新东方] 提供的实习机会和实际应用场景，
    
    感谢指导老师和同事们的支持与建议，
    
    感谢开源社区提供的优秀技术栈和工具。
  ]
]

#v(30pt)

#align(center)[
  #line(length: 60%, stroke: rgb("#dee2e6"))
  
  #v(10pt)
  

  
  #text(10pt, fill: rgb("#6c757d"))[
    文档版本: v1.0 | 更新日期: 2025年10月 | 作者: 王浩然
  ]
]