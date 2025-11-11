import { describe, it, expect, vi, beforeEach } from "vitest";
import {
  showErrorToast,
  showSuccessToast,
  showWarningToast,
  showInfoToast,
  handleAsync,
  getErrorMessage,
} from "../errorHandling";

// Mock the global showToast function
(global as any).showToast = vi.fn();

describe("errorHandling", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("showErrorToast", () => {
    it("detects network errors", () => {
      const error = new Error("Network connection failed");
      showErrorToast(error);

      expect((global as any).showToast).toHaveBeenCalledWith(
        expect.objectContaining({
          type: "error",
          title: "Network Error",
        })
      );
    });

    it("detects file not found errors", () => {
      const error = new Error("File not found at /path/to/file");
      showErrorToast(error);

      expect((global as any).showToast).toHaveBeenCalledWith(
        expect.objectContaining({
          type: "error",
          title: "File Not Found",
        })
      );
    });

    it("detects Drive connection errors", () => {
      const error = new Error("Google Drive not connected");
      showErrorToast(error);

      expect((global as any).showToast).toHaveBeenCalledWith(
        expect.objectContaining({
          type: "error",
          title: "Google Drive Not Connected",
        })
      );
    });

    it("detects token expiry errors", () => {
      const error = new Error("Token has expired");
      showErrorToast(error);

      expect((global as any).showToast).toHaveBeenCalledWith(
        expect.objectContaining({
          type: "error",
          title: "Session Expired",
        })
      );
    });

    it("detects backup errors", () => {
      const error = new Error("Backup operation failed");
      showErrorToast(error);

      expect((global as any).showToast).toHaveBeenCalledWith(
        expect.objectContaining({
          type: "error",
          title: "Backup Failed",
        })
      );
    });

    it("detects restore errors", () => {
      const error = new Error("Restore operation failed");
      showErrorToast(error);

      expect((global as any).showToast).toHaveBeenCalledWith(
        expect.objectContaining({
          type: "error",
          title: "Restore Failed",
        })
      );
    });

    it("detects validation errors", () => {
      const error = new Error("Required field is missing");
      showErrorToast(error);

      expect((global as any).showToast).toHaveBeenCalledWith(
        expect.objectContaining({
          type: "error",
          title: "Validation Error",
        })
      );
    });

    it("detects database errors", () => {
      const error = new Error("Database operation failed");
      showErrorToast(error);

      expect((global as any).showToast).toHaveBeenCalledWith(
        expect.objectContaining({
          type: "error",
          title: "Database Error",
        })
      );
    });

    it("handles unknown errors", () => {
      const error = new Error("Something weird happened");
      showErrorToast(error);

      expect((global as any).showToast).toHaveBeenCalledWith(
        expect.objectContaining({
          type: "error",
          title: "Unexpected Error",
        })
      );
    });

    it("includes error context details", () => {
      const error = new Error("Some error");
      const context = {
        operation: "creating backup",
        details: "Additional info",
        suggestion: "Try again",
      };

      showErrorToast(error, context);

      expect((global as any).showToast).toHaveBeenCalledWith(
        expect.objectContaining({
          type: "error",
          message: expect.stringContaining("Additional info"),
        })
      );
    });

    it("includes suggestions in error message", () => {
      const error = new Error("Network error");
      showErrorToast(error);

      expect((global as any).showToast).toHaveBeenCalledWith(
        expect.objectContaining({
          message: expect.stringContaining("💡"),
        })
      );
    });

    it("handles non-Error objects", () => {
      const error = "String error message";
      showErrorToast(error);

      expect((global as any).showToast).toHaveBeenCalled();
    });
  });

  describe("showSuccessToast", () => {
    it("shows success toast with default duration", () => {
      showSuccessToast("Success", "Operation completed");

      expect((global as any).showToast).toHaveBeenCalledWith({
        type: "success",
        title: "Success",
        message: "Operation completed",
        duration: 5000,
      });
    });

    it("shows success toast with custom duration", () => {
      showSuccessToast("Success", "Operation completed", 3000);

      expect((global as any).showToast).toHaveBeenCalledWith({
        type: "success",
        title: "Success",
        message: "Operation completed",
        duration: 3000,
      });
    });
  });

  describe("showWarningToast", () => {
    it("shows warning toast", () => {
      showWarningToast("Warning", "Please be careful");

      expect((global as any).showToast).toHaveBeenCalledWith({
        type: "warning",
        title: "Warning",
        message: "Please be careful",
        duration: 5000,
      });
    });
  });

  describe("showInfoToast", () => {
    it("shows info toast", () => {
      showInfoToast("Info", "Here's some information");

      expect((global as any).showToast).toHaveBeenCalledWith({
        type: "info",
        title: "Info",
        message: "Here's some information",
        duration: 5000,
      });
    });
  });

  describe("handleAsync", () => {
    it("returns result on success", async () => {
      const operation = vi.fn().mockResolvedValue("success");
      const result = await handleAsync(operation, {
        operation: "test operation",
      });

      expect(result).toBe("success");
      expect((global as any).showToast).not.toHaveBeenCalled();
    });

    it("returns null and shows toast on error", async () => {
      const operation = vi.fn().mockRejectedValue(new Error("Failed"));
      const result = await handleAsync(operation, {
        operation: "test operation",
      });

      expect(result).toBeNull();
      expect((global as any).showToast).toHaveBeenCalled();
    });

    it("passes context to error toast", async () => {
      const operation = vi.fn().mockRejectedValue(new Error("Failed"));
      const context = {
        operation: "creating backup",
        details: "Disk full",
      };

      await handleAsync(operation, context);

      expect((global as any).showToast).toHaveBeenCalledWith(
        expect.objectContaining({
          type: "error",
        })
      );
    });
  });

  describe("getErrorMessage", () => {
    it("extracts message from Error object", () => {
      const error = new Error("Test error message");
      expect(getErrorMessage(error)).toBe("Test error message");
    });

    it("converts non-Error to string", () => {
      expect(getErrorMessage("string error")).toBe("string error");
      expect(getErrorMessage(123)).toBe("123");
      expect(getErrorMessage({ msg: "error" })).toContain("[object Object]");
    });

    it("handles null and undefined", () => {
      expect(getErrorMessage(null)).toBe("null");
      expect(getErrorMessage(undefined)).toBe("undefined");
    });
  });

  describe("error detection edge cases", () => {
    it("prioritizes more specific error types", () => {
      // "Drive not connected" should be detected as drive error first
      const error = new Error("Google Drive not connected");
      showErrorToast(error);

      expect((global as any).showToast).toHaveBeenCalledWith(
        expect.objectContaining({
          title: "Google Drive Not Connected",
        })
      );
    });

    it("handles case-insensitive detection", () => {
      const error = new Error("NETWORK ERROR");
      showErrorToast(error);

      expect((global as any).showToast).toHaveBeenCalledWith(
        expect.objectContaining({
          title: "Network Error",
        })
      );
    });
  });

  describe("fallback behavior", () => {
    it("falls back to console if showToast is not available", () => {
      const originalShowToast = (global as any).showToast;
      delete (global as any).showToast;

      const consoleSpy = vi.spyOn(console, "error").mockImplementation(() => {});

      const error = new Error("Test error");
      showErrorToast(error);

      expect(consoleSpy).toHaveBeenCalled();

      consoleSpy.mockRestore();
      (global as any).showToast = originalShowToast;
    });
  });
});
