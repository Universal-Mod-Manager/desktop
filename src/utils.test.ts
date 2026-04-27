import { describe, it, expect } from "vitest";

const mockApiCall = (success: boolean) =>
  new Promise((resolve, reject) => {
    setTimeout(
      () => (success ? resolve({ status: "ok" }) : reject("error")),
      50,
    );
  });

describe("Generic CI Suite (Frontend)", () => {
  it("should handle string manipulations", () => {
    const greeting = "Hello Tauri";
    expect(greeting).toContain("Tauri");
    expect(greeting.toLowerCase()).toEqual("hello tauri");
  });

  it("should handle asynchronous code (Promises)", async () => {
    const result = await mockApiCall(true);
    expect(result).toEqual({ status: "ok" });
  });
});
