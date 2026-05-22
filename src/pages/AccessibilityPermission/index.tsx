import { invoke } from "@tauri-apps/api/core";
import { Button } from "@heroui/react";

export function AccessibilityPermissionPage() {
  const openSettings = () => {
    invoke("open_accessibility_settings").catch((error) => {
      console.warn("Failed to open Accessibility settings", error);
    });
  };

  const dismiss = () => {
    invoke("dismiss_accessibility_permission_prompt").catch((error) => {
      console.warn("Failed to dismiss Accessibility permission prompt", error);
    });
  };

  return (
    <main className="flex min-h-screen items-center justify-center bg-background p-5">
      <section className="w-full max-w-sm rounded-lg border border-default-200 bg-content1 p-5 shadow-sm">
        <div className="space-y-3">
          <div>
            <h1 className="text-lg font-semibold text-foreground">开启辅助功能权限</h1>
            <p className="mt-2 text-sm leading-6 text-default-600">
              划词弹窗需要 macOS 辅助功能权限，授权后 Bugoo 才能读取你在其他应用中选中的文本。
            </p>
          </div>
          <div className="flex justify-end gap-2 pt-2">
            <Button variant="outline" onPress={dismiss}>
              稍后
            </Button>
            <Button onPress={openSettings}>去系统设置</Button>
          </div>
        </div>
      </section>
    </main>
  );
}
