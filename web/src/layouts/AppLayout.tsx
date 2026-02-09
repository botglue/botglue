import { Outlet, Link, useLocation } from "react-router";

const navItems = [
  { path: "/", label: "Dashboard" },
  { path: "/settings", label: "Settings" },
];

export function AppLayout() {
  const location = useLocation();

  return (
    <div className="min-h-screen bg-[#0a0a0f] text-[#f0f0f5]">
      <header className="border-b border-[#1a1a2f] px-6 py-3 flex items-center gap-8">
        <Link to="/" className="text-xl font-semibold">
          BotGlue
        </Link>
        <nav className="flex gap-4">
          {navItems.map((item) => (
            <Link
              key={item.path}
              to={item.path}
              className={`text-sm ${
                location.pathname === item.path
                  ? "text-[#f0f0f5]"
                  : "text-[#6b6b7b] hover:text-[#a0a0b0]"
              }`}
            >
              {item.label}
            </Link>
          ))}
        </nav>
      </header>
      <main className="px-6 py-6">
        <Outlet />
      </main>
    </div>
  );
}
