import type { Metadata } from "next";
import "./globals.css";
import { WalletContextProvider } from "./components/WalletContextProvider";

export const metadata: Metadata = {
  title: "Solana App",
  description: "Solana wallet integration",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body>
        <WalletContextProvider>
          {children}
        </WalletContextProvider>
      </body>
    </html>
  );
}
