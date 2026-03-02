"use client";

import {
  createContext,
  useContext,
  useEffect,
  useState,
  useCallback,
  type ReactNode,
} from "react";
import { api } from "./api";

interface User {
  user_id: string;
  username: string;
}

interface AuthContextType {
  user: User | null;
  loading: boolean;
  login: (username: string, password: string) => Promise<void>;
  register: (username: string, password: string) => Promise<void>;
  logout: () => void;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User | null>(null);
  const [loading, setLoading] = useState(true);

  // Check for existing session on mount
  useEffect(() => {
    const checkAuth = async () => {
      try {
        const me = await api.me();
        if (me) {
          setUser(me);
        }
      } catch {
        localStorage.removeItem("notice_token");
      } finally {
        setLoading(false);
      }
    };

    checkAuth();
  }, []);

  const login = useCallback(async (username: string, password: string) => {
    const res = await api.login(username, password);
    localStorage.setItem("notice_token", res.token);
    setUser({ user_id: res.user_id, username: res.username });
  }, []);

  const register = useCallback(
    async (username: string, password: string) => {
      const res = await api.register(username, password);
      localStorage.setItem("notice_token", res.token);
      setUser({ user_id: res.user_id, username: res.username });
    },
    []
  );

  const logout = useCallback(() => {
    localStorage.removeItem("notice_token");
    setUser(null);
  }, []);

  return (
    <AuthContext.Provider value={{ user, loading, login, register, logout }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error("useAuth must be used within an AuthProvider");
  }
  return context;
}
