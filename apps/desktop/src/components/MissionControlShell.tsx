import type { ChatResponse, HealthSnapshot } from "@adaptive/shared-types";
import { MissionControlLayout, Panel } from "@adaptive/ui";

type MissionControlShellProps = {
  requestText: string;
  setRequestText: (next: string) => void;
  onSubmit: () => void;
  status: string;
  response: ChatResponse | null;
  health: HealthSnapshot | null;
};

export function MissionControlShell({
  requestText,
  setRequestText,
  onSubmit,
  status,
  response,
  health,
}: MissionControlShellProps) {
  return (
    <MissionControlLayout
      leftRail={
        <>
          <Panel title="Mode" subtitle="Research Control">
            <div className="metric-stack">
              <div className="metric-card">
                <span>Current mode</span>
                <strong>Research</strong>
              </div>
              <div className="metric-card">
                <span>Approvals armed</span>
                <strong>Risk Gate On</strong>
              </div>
            </div>
          </Panel>
          <Panel title="Watchlists" subtitle="Priority Markets">
            <ul className="watchlist">
              <li>BTC-USD</li>
              <li>ETH-USD</li>
              <li>ES1!</li>
              <li>NVDA</li>
            </ul>
          </Panel>
        </>
      }
      mainStage={
        <>
          <Panel
            title="Operator Chat"
            subtitle="Conversational control surface"
            aside={<span className="status-pill">{status}</span>}
          >
            <div className="chat-grid">
              <textarea
                value={requestText}
                onChange={(event) => setRequestText(event.target.value)}
                placeholder="Ask for market structure, signal synthesis, or a risk-reviewed thesis."
              />
              <button className="primary-button" onClick={onSubmit}>
                Run Operator
              </button>
            </div>
          </Panel>
          <Panel title="Fused Thesis" subtitle="Consensus output">
            {response ? (
              <div className="thesis-card">
                <h3>{response.thesis.market}</h3>
                <p>{response.thesis.thesis}</p>
                <div className="tag-row">
                  <span>{response.thesis.direction}</span>
                  <span>{response.thesis.regime}</span>
                  <span>{response.thesis.confidence.toFixed(2)} confidence</span>
                  <span>
                    {response.thesis.disagreementScore.toFixed(2)} disagreement
                  </span>
                </div>
                <p className="decision-copy">{response.decisionSummary}</p>
              </div>
            ) : (
              <p className="muted-copy">
                No thesis yet. Dispatch a request to the orchestrator.
              </p>
            )}
          </Panel>
        </>
      }
      rightRail={
        <>
          <Panel title="Runtime" subtitle="Health and telemetry">
            {health ? (
              <div className="metric-stack">
                <div className="metric-card">
                  <span>Service</span>
                  <strong>{health.service}</strong>
                </div>
                <div className="metric-card">
                  <span>Healthy</span>
                  <strong>{health.healthy ? "Yes" : "No"}</strong>
                </div>
              </div>
            ) : (
              <p className="muted-copy">Health snapshot unavailable.</p>
            )}
          </Panel>
          <Panel title="Dissent" subtitle="Why not">
            {response?.thesis.whyNot.length ? (
              <ul className="watchlist">
                {response.thesis.whyNot.map((item) => (
                  <li key={item}>{item}</li>
                ))}
              </ul>
            ) : (
              <p className="muted-copy">
                No dissent captured for the current thesis.
              </p>
            )}
          </Panel>
        </>
      }
      footer={
        <div className="footer-strip">
          <span>Adaptive routing</span>
          <span>Policy-governed connectors</span>
          <span>Tamper-evident audit log</span>
        </div>
      }
    />
  );
}

