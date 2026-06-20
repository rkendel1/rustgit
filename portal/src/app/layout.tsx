import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "TryThisSoftware Portal",
  description: "Next.js portal for managing executions, workspaces, and platform services.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
