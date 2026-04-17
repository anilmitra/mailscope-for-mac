import { Component, type ReactNode } from "react";

interface Props {
  children: ReactNode;
}

interface State {
  error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  render() {
    const { error } = this.state;
    if (error) {
      return (
        <div
          style={{
            padding: "2rem",
            color: "#ff453a",
            fontFamily: "monospace",
            whiteSpace: "pre-wrap",
            background: "#1c1c1e",
            height: "100vh",
            overflowY: "auto",
          }}
        >
          <strong style={{ fontSize: "1rem" }}>Runtime error</strong>
          {"\n\n"}
          {error.message}
          {"\n\n"}
          {error.stack}
        </div>
      );
    }
    return this.props.children;
  }
}
