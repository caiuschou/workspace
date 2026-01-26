import Link from "next/link";

interface DocCardProps {
  title: string;
  description: string;
  href: string;
  icon?: React.ReactNode;
}

export function DocCard({ title, description, href, icon }: DocCardProps) {
  return (
    <Link
      href={href}
      className="block p-6 rounded-xl bg-gray-800/50 border border-gray-700 hover:border-blue-500 hover:bg-gray-800 transition-all group"
    >
      <div className="flex items-start gap-4">
        {icon && (
          <div className="text-2xl text-gray-400 group-hover:text-blue-400 transition-colors">
            {icon}
          </div>
        )}
        <div>
          <h3 className="text-lg font-semibold text-white group-hover:text-blue-400 transition-colors mb-1">
            {title}
          </h3>
          <p className="text-gray-400 text-sm">{description}</p>
        </div>
      </div>
    </Link>
  );
}
