import { ThemeProvider } from "next-themes";
import { ReactNode } from "react";

export default function ThemesProvider({ children }: { children: ReactNode }) {
  return (
    <ThemeProvider enableSystem attribute={"class"} defaultTheme="system">
      {children}
    </ThemeProvider>
  );
}
