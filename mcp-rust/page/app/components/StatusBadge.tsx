type Status = "completed" | "in_progress" | "not_started" | "partial";

interface StatusBadgeProps {
  status: Status;
  label?: string;
}

const statusConfig: Record<Status, { bg: string; text: string; defaultLabel: string }> = {
  completed: {
    bg: "bg-green-500/20",
    text: "text-green-400",
    defaultLabel: "完成",
  },
  in_progress: {
    bg: "bg-yellow-500/20",
    text: "text-yellow-400",
    defaultLabel: "进行中",
  },
  partial: {
    bg: "bg-yellow-500/20",
    text: "text-yellow-400",
    defaultLabel: "部分完成",
  },
  not_started: {
    bg: "bg-gray-500/20",
    text: "text-gray-400",
    defaultLabel: "未开始",
  },
};

export function StatusBadge({ status, label }: StatusBadgeProps) {
  const config = statusConfig[status];
  return (
    <span
      className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${config.bg} ${config.text}`}
    >
      {label || config.defaultLabel}
    </span>
  );
}

export function StatusIcon({ status }: { status: Status }) {
  switch (status) {
    case "completed":
      return <span className="text-green-400">✓</span>;
    case "in_progress":
    case "partial":
      return <span className="text-yellow-400">◐</span>;
    case "not_started":
      return <span className="text-gray-400">○</span>;
  }
}
