import type { PropsWithChildren, ReactNode } from "react";

type PanelProps = PropsWithChildren<{
  title: string;
  subtitle?: string;
  aside?: ReactNode;
}>;

export function Panel({ title, subtitle, aside, children }: PanelProps) {
  return (
    <section className="panel">
      <header className="panel-header">
        <div>
          <p className="panel-eyebrow">{title}</p>
          {subtitle ? <h2>{subtitle}</h2> : null}
        </div>
        {aside ? <div>{aside}</div> : null}
      </header>
      <div className="panel-body">{children}</div>
    </section>
  );
}

type MissionControlLayoutProps = {
  leftRail: ReactNode;
  mainStage: ReactNode;
  rightRail: ReactNode;
  footer?: ReactNode;
};

export function MissionControlLayout({
  leftRail,
  mainStage,
  rightRail,
  footer,
}: MissionControlLayoutProps) {
  return (
    <div className="mission-control-shell">
      <aside className="left-rail">{leftRail}</aside>
      <main className="main-stage">{mainStage}</main>
      <aside className="right-rail">{rightRail}</aside>
      {footer ? <footer className="bottom-strip">{footer}</footer> : null}
    </div>
  );
}

