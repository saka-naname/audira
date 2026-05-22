# Issue Workflow

## 基本方針

- Epicは親Issueとして作成する
- User StoryはEpicのSub-issueとして作成する
- TaskはStory内チェックリストとして管理する
- BugとResearchは通常Issueとして作成する
- PRは原則としてUser Story単位で作成する

## 使用するラベル

```txt
type: epic
type: story
type: bug
type: research
```

## 階層

```txt
Epic Issue
└── User Story Sub-issues
    └── Tasks: checklist in Story issue
```

## Issue種別

### Epic

大きな機能領域・開発テーマを表す親Issue。

### User Story

ユーザー価値単位の作業Issue。EpicのSub-issueとして作成する。

### Task

個別Issueにはしない。Story内チェックリストとして管理する。

### Bug

不具合を管理する通常Issue。

### Research / Spike

実装前の調査・検証を管理する通常Issue。

## Doneの基準

StoryをDoneにするには、以下を満たすこと。

- Acceptance Criteriaを満たしている
- 手元で動作確認している
- 必要なテストを追加・更新している
- 必要なドキュメントを更新している
- PRでセルフレビューしている
