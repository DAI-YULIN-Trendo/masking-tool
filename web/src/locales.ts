export type Language = 'en' | 'ja' | 'zh';

export const translations = {
    ja: {
        title: "PDF セキュア墨消しツール",
        subtitle: "企業が外部へデータを提供する際の、最も安全な選択肢",
        helpButton: "使い方ガイド",
        helpTitle: "使い方ガイド（マニュアル）",
        helpClose: "閉じる",

        // Manual Sections
        step1Title: "1. ファイルの読み込み",
        step1Desc: "画面中央の点線枠をクリックしてPDFファイルを選択するか、ファイルを直接ドラッグ＆ドロップしてください。パスワード付きPDFも対応しています。",
        step2Title: "2. 黒塗り・白塗りの作成",
        step2Desc1: "ツールバーで「黒塗り」または「白塗り」を選択します。",
        step2Desc2: "隠したい部分をマウスでドラッグすると、その範囲が塗りつぶされます。",
        step3Title: "3. 編集と調整",
        step3Desc1: "「移動」ツールを選ぶか、既存のブロックをクリックすると選択状態（青枠）になります。",
        step3Desc2: "選択したブロックはドラッグして位置を調整できます。",
        step3Desc3: "「全ページ適用」ボタンを押すと、選択中のブロックと同じ位置・大きさのブロックを全てのページにコピーします。",
        step3Desc4: "間違えた場合は「戻る (Undo)」ボタンや「削除」ボタンを使用してください。",
        step4Title: "4. 保存",
        step4Desc: "全ての作業が終わったら、画面下の「墨消し実行 (PDF保存)」ボタンを押してください。ファイル名に _masked が付加されて保存されます。",

        securityNoteTitle: "セキュリティについて:",
        securityNoteDesc: "本ツールは完全にオフラインで動作します。アップロードしたファイルが外部サーバーに送信されることはありません。企業のコンプライアンス基準に準拠した、物理的なデータ削除を行います。",

        // UI Elements
        dropZoneText: "PDFファイルをここにドラッグ＆ドロップ",
        dropZoneSub: "またはクリックしてファイルを選択",
        resetFile: "ファイルを再選択 (最初に戻る)",
        confirmReset: "編集内容は破棄されます。よろしいですか？",

        toolBlack: "黒塗り",
        toolWhite: "白塗り",
        toolMove: "移動",

        actionUndo: "元に戻す",
        actionRedo: "やり直し",
        actionApplyAll: "全ページ適用",
        actionDelete: "削除",
        actionDeleteKey: "削除",

        page: "ページ",

        saveButton: "墨消し実行 (PDF保存)",
        processing: "処理中...",

        statusMasks: "現在のマスク数: {count} 箇所",

        // Alerts & Prompts
        passwordPrompt: "PDFのパスワードを入力してください:",
        passwordIncorrect: "パスワードが間違っています。再入力してください。",
        saveTitle: "保存しました！",
        saveMessage: "保存先: {path}\n\n最初の画面（ファイル選択）に戻りますか？\n[キャンセル] を押すと編集を続けます。",
        saveError: "保存エラー(詳細): {error}",
        loadError: "PDFの読み込みに失敗しました。",
        applyAllAlert: "全ページ ({total}ページ) に同じ記号を適用しました。",

        openSourceNote: "オープンソース精神に基づき、処理の透明性を保証しています。"
    },
    en: {
        title: "PDF Secure Masker",
        subtitle: "The safest choice for enterprises sharing data externally.",
        helpButton: "User Guide",
        helpTitle: "User Guide (Manual)",
        helpClose: "Close",

        step1Title: "1. Load File",
        step1Desc: "Click the dotted area to select a PDF or drag & drop directly. Password-protected PDFs are supported.",
        step2Title: "2. Create Masks",
        step2Desc1: "Select 'Black Mask' or 'White Mask' from the toolbar.",
        step2Desc2: "Drag over the area you want to hide.",
        step3Title: "3. Edit & Adjust",
        step3Desc1: "Select 'Move' tool or click an existing block to select it (blue border).",
        step3Desc2: "Drag selected blocks to adjust position.",
        step3Desc3: "Clicking 'Apply to All Pages' copies the selected mask to the same position on every page.",
        step3Desc4: "Use 'Undo' or 'Delete' to correct mistakes.",
        step4Title: "4. Save",
        step4Desc: "When finished, click 'Execute Redaction (Save PDF)'. The file will be saved with a _masked suffix.",

        securityNoteTitle: "Security Note:",
        securityNoteDesc: "This tool operates 100% offline. No files are sent to external servers. It performs physical data deletion specifically for enterprise compliance.",

        dropZoneText: "Drag & Drop PDF here",
        dropZoneSub: "or click to select file",
        resetFile: "Select New File (Reset)",
        confirmReset: "Edits will be discarded. Are you sure?",

        toolBlack: "Black Mask",
        toolWhite: "White Mask",
        toolMove: "Move",

        actionUndo: "Undo",
        actionRedo: "Redo",
        actionApplyAll: "Apply to All",
        actionDelete: "Delete",
        actionDeleteKey: "Delete",

        page: "Page",

        saveButton: "Execute Redaction (Save PDF)",
        processing: "Processing...",

        statusMasks: "Masks: {count}",

        passwordPrompt: "Please enter PDF password:",
        passwordIncorrect: "Incorrect password. Please try again.",
        saveTitle: "Saved Successfully!",
        saveMessage: "Saved to: {path}\n\nReturn to start screen?\nClick [Cancel] to continue editing.",
        saveError: "Save Error: {error}",
        loadError: "Failed to load PDF.",
        applyAllAlert: "Applied mask to all pages ({total} pages).",

        openSourceNote: "Transparent processing based on the Open Source spirit."
    },
    zh: {
        title: "PDF 敏感信息与隐私保护工具",
        subtitle: "企业级数据合规与安全脱敏的本地化首选方案",
        helpButton: "使用说明",
        helpTitle: "操作指南",
        helpClose: "关闭",

        step1Title: "1. 导入文件",
        step1Desc: "点击虚线区域选择 PDF 文件，或直接将文件拖拽至此。支持已加密的 PDF 文档。",
        step2Title: "2. 创建遮盖区域",
        step2Desc1: "在上方工具栏选择「黑色遮盖」或「白色遮盖」。",
        step2Desc2: "在页面上按住鼠标左键并拖拽，框选需要隐藏的敏感内容。",
        step3Title: "3. 编辑与调整",
        step3Desc1: "使用「移动选区」工具，或点击已有的遮盖块选中它（呈现蓝框）。",
        step3Desc2: "拖拽遮盖块可微调其位置。",
        step3Desc3: "点击「应用至全文档」按钮，可将当前遮盖块批量复制到每一页的相同位置（适用于页眉/页脚）。",
        step3Desc4: "如操作有误，请使用「撤销 (Undo)」或「删除选中」按钮。",
        step4Title: "4. 安全导出",
        step4Desc: "确认无误后，点击底部的「执行脱敏保护 (导出文件)」按钮。生成的文件将自动添加 _masked 后缀。",

        securityNoteTitle: "安全与合规声明:",
        securityNoteDesc: "本工具采用 100% 离线运行模式，您的任何文件都不会上传至外部服务器。系统将执行物理级的数据销毁操作，确保被遮盖的信息无法被任何技术手段恢复，完全符合企业级的合规与审计标准。",

        dropZoneText: "将 PDF 文件拖拽至此",
        dropZoneSub: "或点击此处选择文件",
        resetFile: "重新选择文件",
        confirmReset: "当前编辑内容将丢失，是否确认返回？",

        toolBlack: "黑色遮盖",
        toolWhite: "白色遮盖",
        toolMove: "移动选区",

        actionUndo: "撤销操作",
        actionRedo: "恢复操作",
        actionApplyAll: "应用至全文档",
        actionDelete: "删除选中",
        actionDeleteKey: "删除",

        page: "页码",

        saveButton: "执行脱敏保护 (导出文件)",
        processing: "正在处理中...",

        statusMasks: "已添加遮盖: {count} 处",

        passwordPrompt: "该文档已加密，请输入密码:",
        passwordIncorrect: "密码错误，请重试。",
        saveTitle: "导出成功！",
        saveMessage: "文件已保存至: {path}\n\n是否返回主界面？\n点击 [取消] 可继续编辑当前文档。",
        saveError: "保存失败 (错误信息): {error}",
        loadError: "无法加载 PDF 文档。",
        applyAllAlert: "已将该遮盖应用到全文档共 {total} 页。",

        openSourceNote: "基于开源精神构建，确保数据处理过程透明、可信、无后门。"
    }
};
