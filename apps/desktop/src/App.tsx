import { useEffect, useRef, useState } from "react";

import type { ChatResponse, HealthSnapshot } from "@adaptive/shared-types";
import { OperatorSdk } from "@adaptive/sdk";

import { MissionControlShell } from "./components/MissionControlShell";
import { DesktopCommandBus } from "./lib/commandBus";

const defaultPrompt =
  "Synthesize a 1-day thesis on BTC-USD with regime context, disagreement view, and risk constraints.";

export default function App() {
  const sdkRef = useRef<OperatorSdk | null>(null);
  const commandBusRef = useRef<DesktopCommandBus | null>(null);
  const [requestText, setRequestText] = useState(defaultPrompt);
  const [status, setStatus] = useState("idle");
  const [response, setResponse] = useState<ChatResponse | null>(null);
  const [health, setHealth] = useState<HealthSnapshot | null>(null);

  const sdk = sdkRef.current ?? (sdkRef.current = new OperatorSdk());
  const commandBus =
    commandBusRef.current ?? (commandBusRef.current = new DesktopCommandBus());

  useEffect(() => {
    let mounted = true;
    sdk
      .health()
      .then((snapshot) => {
        if (mounted) {
          setHealth(snapshot);
        }
      })
      .catch(() => {
        if (mounted) {
          setHealth(null);
        }
      });

    const unsubscribe = commandBus.subscribe((event) => {
      if (event.type === "status") {
        setStatus(event.status);
      }
      if (event.type === "response") {
        setResponse(event.response);
      }
      if (event.type === "error") {
        setStatus(event.message);
      }
    });

    return () => {
      mounted = false;
      unsubscribe();
    };
  }, []);

  async function onSubmit() {
    await commandBus.dispatchChat({
      actor: "desktop-operator",
      mode: "research",
      market: "BTC-USD",
      message: requestText,
      watchlist: ["BTC-USD", "ETH-USD", "ES1!"],
    });
  }

  return (
    <MissionControlShell
      requestText={requestText}
      setRequestText={setRequestText}
      onSubmit={() => {
        void onSubmit();
      }}
      status={status}
      response={response}
      health={health}
    />
  );
}
