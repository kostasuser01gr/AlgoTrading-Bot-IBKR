import type { ChatRequest, ChatResponse } from "@adaptive/shared-types";
import { OperatorSdk } from "@adaptive/sdk";

type CommandStatus = "idle" | "running" | "completed" | "failed";

export type CommandEvent =
  | { type: "status"; status: CommandStatus }
  | { type: "response"; response: ChatResponse }
  | { type: "error"; message: string };

type Subscriber = (event: CommandEvent) => void;

export class DesktopCommandBus {
  private sdk = new OperatorSdk();
  private subscribers = new Set<Subscriber>();

  subscribe(subscriber: Subscriber) {
    this.subscribers.add(subscriber);
    return () => this.subscribers.delete(subscriber);
  }

  async dispatchChat(request: ChatRequest) {
    this.emit({ type: "status", status: "running" });
    try {
      const response = await this.sdk.chat(request);
      this.emit({ type: "response", response });
      this.emit({ type: "status", status: "completed" });
      return response;
    } catch (error) {
      const message =
        error instanceof Error ? error.message : "unknown command bus error";
      this.emit({ type: "error", message });
      this.emit({ type: "status", status: "failed" });
      throw error;
    }
  }

  private emit(event: CommandEvent) {
    for (const subscriber of this.subscribers) {
      subscriber(event);
    }
  }
}

