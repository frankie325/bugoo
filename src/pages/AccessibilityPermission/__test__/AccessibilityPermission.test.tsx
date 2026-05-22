import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { AccessibilityPermissionPage } from "../index";

const invokeMock = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

afterEach(() => {
  cleanup();
  invokeMock.mockReset();
});

describe("AccessibilityPermissionPage", () => {
  it("renders permission instructions", () => {
    render(<AccessibilityPermissionPage />);

    expect(screen.getByText("开启辅助功能权限")).toBeTruthy();
    expect(screen.getByText(/划词弹窗需要 macOS 辅助功能权限/)).toBeTruthy();
  });

  it("opens system settings", () => {
    invokeMock.mockResolvedValueOnce(undefined);
    render(<AccessibilityPermissionPage />);

    fireEvent.click(screen.getByRole("button", { name: "去系统设置" }));

    expect(invokeMock).toHaveBeenCalledWith("open_accessibility_settings");
  });

  it("dismisses the prompt", () => {
    invokeMock.mockResolvedValueOnce(undefined);
    render(<AccessibilityPermissionPage />);

    fireEvent.click(screen.getByRole("button", { name: "稍后" }));

    expect(invokeMock).toHaveBeenCalledWith("dismiss_accessibility_permission_prompt");
  });
});
