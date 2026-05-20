# 本地词典与多翻译引擎功能设计

日期：2026-05-20

## 背景

Bugoo 当前已经有 `TranslationProvider`、`WordInsightProvider`、`translate_text` 命令和翻译设置页。现有翻译引擎主要面向自定义大模型，`deepl`、`google` 仍处于占位状态。

新的目标是把翻译查询拆成两层：

1. 本地词典优先查询，用于获取音标、词性、基础释义和可选例句。
2. 本地词典未命中时，再调用翻译引擎。

第一版不做缓存、不做服务端代理、不内置厂商 API 密钥。

## 目标

- 系统内置 ECDICT 本地词库。
- ECDICT 以只读 SQLite 词库形式随 App 打包。
- 查询翻译时优先查本地词典。
- 本地词典查不到时调用翻译接口。
- 翻译引擎分为三类：
  - 系统内置翻译：LibreTranslate
  - 厂商 API：Google、DeepL、Bing / Microsoft Translator、百度、腾讯、有道
  - 自定义大模型：OpenAI-compatible 自定义接口
- 音标、词性、例句等学习字段如果找不到，则返回空值，不伪造数据。

## 非目标

- 不做翻译缓存。
- 不做服务端 Translation Gateway。
- 不在客户端内置任何厂商 API 密钥。
- 不在第一版本地部署 LibreTranslate。
- 不要求厂商 API 返回音标、词性、例句。
- 不扩展 `WordInsightProvider` 的大模型详情生成逻辑。

## 总体架构

```txt
前端 translate(text, sourceLang, targetLang)
  -> Tauri invoke("translate_text")
    -> DictionaryProvider 查询 ECDICT
      -> 命中：返回 TranslationResult
      -> 未命中：TranslationProvider 调用翻译引擎
```

查询结果继续复用现有 `TranslationResult`：

```ts
type TranslationResult = {
  translation: string;
  detectedSourceLang: string | null;
  phonetic: string | null;
  partOfSpeech: string[];
  definitions: string[];
  examples: TranslationExample[];
};
```

## 本地词典设计

### 词库形态

ECDICT 第一版处理成独立只读 SQLite 文件，随 App 打包。例如：

```txt
src-tauri/resources/dictionaries/ecdict.sqlite
```

应用运行时只读打开该词库，不把 ECDICT 导入用户主数据库。

### 设计理由

- 主业务数据库保持轻量。
- 词库升级可以通过替换资源文件完成。
- 查询逻辑独立，减少和用户数据迁移的耦合。
- 不需要首次启动导入，避免启动耗时。

### 词典 Provider

新增后端端口：

```rust
DictionaryProvider
DictionaryLookupRequest
DictionaryLookupResult
```

`DictionaryLookupResult` 字段：

```rust
word: String
translation: String
phonetic: Option<String>
part_of_speech: Vec<String>
definitions: Vec<String>
examples: Vec<TranslationExample>
```

第一版 ECDICT 如果没有例句字段，则 `examples` 返回空数组。

## 查询规则

### 是否查询词典

第一版采用保守规则：

- 输入是单个单词：优先查 ECDICT。
- 输入是短语：可先尝试查 ECDICT，未命中走翻译引擎。
- 输入明显是句子：跳过 ECDICT，直接走翻译引擎。

句子判断可用简单启发式：

- 包含明显句末标点。
- 分词后超过 4 个 token。
- 文本长度超过 48 个字符。

这些阈值只用于第一版本地词典查询分流，后续可以根据真实使用反馈调整。

### 命中本地词典

返回：

```txt
translation = ECDICT 中文释义或主要释义
detectedSourceLang = sourceLang 或 "en"
phonetic = ECDICT 音标
partOfSpeech = ECDICT 词性
definitions = ECDICT 释义拆分结果
examples = ECDICT 可用例句，否则 []
```

### 未命中本地词典

调用当前设置页选择的翻译引擎。

翻译引擎返回的学习字段规则：

```txt
phonetic = null
partOfSpeech = []
definitions = []
examples = []
```

除非某个引擎官方 API 明确返回这些字段，否则不尝试猜测。

## 翻译引擎分类

设置字段仍使用：

```txt
translationEngine
```

引擎值：

```txt
libretranslate
google
deepl
microsoft
baidu
tencent
youdao
custom
```

### 系统内置翻译

`libretranslate`

第一版通过 LibreTranslate HTTP API 调用远端或用户自部署实例。

配置字段：

```txt
apiEndpoint   LibreTranslate 服务地址，默认可为空
apiKey        可选
```

如果 `apiEndpoint` 为空，第一版返回明确错误，提示用户在设置页填写 LibreTranslate Endpoint。后续如果项目决定提供默认公共实例，再单独设计。

### 厂商 API

厂商 API 第一版全部采用 BYOK。用户在设置页填写自己的密钥，Bugoo 不承担调用成本，也不在客户端内置密钥。

引擎配置：

```txt
google:
  apiKey = Google Cloud Translation API Key
  apiEndpoint = 可选

deepl:
  apiKey = DeepL Auth Key
  apiEndpoint = 可选

microsoft:
  apiKey = Azure Translator Key
  apiRegion = Azure Translator Region
  apiEndpoint = 可选

baidu:
  apiKey = App ID
  apiSecret = Secret Key

tencent:
  apiKey = Secret ID
  apiSecret = Secret Key
  apiRegion = Region

youdao:
  apiKey = App Key
  apiSecret = App Secret
```

设置页展示名：

```txt
microsoft = Bing / Microsoft Translator
```

### 自定义大模型

`custom`

继续复用现有 OpenAI-compatible 自定义大模型能力。

配置字段：

```txt
apiEndpoint
apiKey
translationModel
translationPrompt
wordDetailPrompt
translationTimeoutMs
```

自定义大模型仍然是唯一负责生成记忆技巧、单词详情说明的能力。

## 设置项设计

继续使用 settings key-value 表。

新增或规范字段：

```txt
translationEngine
apiEndpoint
apiKey
apiSecret
apiRegion
translationModel
translationPrompt
wordDetailPrompt
translationTimeoutMs
```

默认值建议：

```txt
translationEngine = libretranslate
apiEndpoint = ""
apiKey = ""
apiSecret = ""
apiRegion = ""
translationModel = ""
translationPrompt = ""
wordDetailPrompt = ""
translationTimeoutMs = "15000"
```

使用 `INSERT OR IGNORE` 初始化，避免覆盖用户已有配置。

## 设置页 UI

翻译引擎下拉按分组展示：

```txt
系统内置
- LibreTranslate

厂商 API
- Google
- DeepL
- Bing / Microsoft Translator
- 百度
- 腾讯
- 有道

自定义
- 自定义大模型
```

字段按引擎动态显示：

```txt
LibreTranslate:
  Endpoint
  API Key（可选）
  Timeout

Google / DeepL:
  API Key
  Endpoint（可选）
  Timeout

Bing / Microsoft Translator:
  API Key
  Region
  Endpoint（可选）
  Timeout

百度 / 腾讯 / 有道:
  API Key
  API Secret
  Region（仅腾讯）
  Timeout

自定义大模型:
  Endpoint
  API Key
  Model
  Translation Prompt
  Word Detail Prompt
  Timeout
```

## 错误处理

### 本地词典错误

ECDICT 查询失败不应阻断翻译。处理方式：

```txt
记录错误
继续调用翻译引擎
```

### 本地词典未命中

这是正常路径，不显示错误。

### 翻译引擎未配置

返回明确错误：

```txt
请在设置页配置翻译服务
```

具体字段错误：

```txt
缺少 API Key
缺少 API Secret
缺少 Endpoint
缺少 Region
```

### 密钥安全

错误信息、日志、前端提示中不得输出完整 `apiKey`、`apiSecret`。

## 数据流

### 单词查询

```txt
用户查询 "hello"
-> ECDICT 命中
-> 返回 translation / phonetic / partOfSpeech / definitions
```

### 句子查询

```txt
用户查询 "I like this app."
-> 判定为句子
-> 跳过 ECDICT
-> 调用 translationEngine
-> 返回 translation
```

### 词典未命中

```txt
用户查询专有名词或新词
-> ECDICT 未命中
-> 调用 translationEngine
-> 返回 translation，学习字段为空
```

## 测试策略

### Rust 单元测试

- ECDICT 查询命中。
- ECDICT 查询未命中。
- 句子判断跳过词典。
- 词典查询失败时继续翻译。
- 每个 provider 缺字段错误明确。
- Provider 错误不泄露密钥。

### 前端类型检查

- `pnpm tsc --noEmit`

### 构建验证

- `pnpm build`
- `cd src-tauri && cargo build`
- `cd src-tauri && cargo test`

## 后续扩展

- LibreTranslate 本地部署或内嵌服务。
- 翻译缓存。
- Tatoeba 例句库。
- ipa-dict 音标补充。
- WordNet 语义关系。
- 服务端 Translation Gateway。

## 决策记录

- 第一版 ECDICT 使用只读 SQLite 随 App 打包。
- 第一版不内置厂商 API 密钥。
- 第一版厂商 API 使用 BYOK。
- 第一版不做缓存。
- 音标、词性、例句查不到时返回空值。
- `microsoft` 作为内部 engine 名，设置页展示为 `Bing / Microsoft Translator`。
