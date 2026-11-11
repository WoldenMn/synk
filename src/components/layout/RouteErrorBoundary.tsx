import { Component, type ErrorInfo, type ReactNode } from "react";
import { QueryErrorResetBoundary } from "@tanstack/react-query";
import { useLocation } from "react-router-dom";
import { TriangleAlert } from "lucide-react";
import { Button } from "@/components/ui/button";
import { formatError } from "@/lib/errors";
import { reportError } from "@/lib/reportError";

interface BoundaryProps {
  onReset: () => void;
  children: ReactNode;
}

interface BoundaryState {
  error: unknown;
  failed: boolean;
}

class Boundary extends Component<BoundaryProps, BoundaryState> {
  state: BoundaryState = { error: null, failed: false };

  static getDerivedStateFromError(error: unknown): BoundaryState {
    return { error, failed: true };
  }

  componentDidCatch(error: unknown, info: ErrorInfo) {
    reportError(error, {
      source: "boundary",
      context: info.componentStack ?? "route",
      toast: false,
    });
  }

  reset = () => {
    this.props.onReset();
    this.setState({ error: null, failed: false });
  };

  render() {
    if (!this.state.failed) return this.props.children;
    return (
      <div
        role="alert"
        className="flex flex-col items-start gap-3 rounded-lg border border-destructive/40 bg-destructive/5 p-6"
      >
        <p className="flex items-center gap-2 font-semibold">
          <TriangleAlert className="h-5 w-5" aria-hidden="true" />
          Impossible d'afficher cette page
        </p>
        <p className="text-sm">{formatError(this.state.error).userMessage}</p>
        <Button onClick={this.reset}>Réessayer</Button>
      </div>
    );
  }
}

export function RouteErrorBoundary({ children }: { children: ReactNode }) {
  const location = useLocation();
  return (
    <QueryErrorResetBoundary>
      {({ reset }) => (
        <Boundary key={location.pathname} onReset={reset}>
          {children}
        </Boundary>
      )}
    </QueryErrorResetBoundary>
  );
}
