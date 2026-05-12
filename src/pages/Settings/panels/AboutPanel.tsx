import { Button } from "@heroui/react";

export function AboutPanel() {
  const version = "1.0.0";

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-lg font-medium mb-4">关于</h2>
        <div className="space-y-6">
          <div className="flex flex-col items-center text-center py-6">
            <div className="w-16 h-16 bg-primary rounded-2xl flex items-center justify-center mb-4">
              <span className="text-2xl text-white">B</span>
            </div>
            <h3 className="text-xl font-semibold">Bugoo</h3>
            <p className="text-sm text-default-500 mt-1">版本 {version}</p>
            <p className="text-xs text-default-400 mt-2">
              划词翻译 · 艾宾浩斯记忆 · 间隔复习
            </p>
          </div>

          <div className="space-y-3">
            <Button
              variant="outline"
              className="w-full justify-start"
              onPress={() => {}}
            >
              检查更新
            </Button>
            <Button
              variant="outline"
              className="w-full justify-start"
              onPress={() => {}}
            >
              意见反馈
            </Button>
            <Button
              variant="outline"
              className="w-full justify-start"
              onPress={() => {}}
            >
              帮助文档
            </Button>
            <Button
              variant="outline"
              className="w-full justify-start"
              onPress={() => {}}
            >
              开源许可
            </Button>
          </div>

          <div className="pt-4 border-t border-default-200">
            <p className="text-xs text-default-400 text-center">
              Made with ❤️ by Bugoo Team
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
