import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { AuthProvider } from "./context/AuthContext";
import { UserDataProvider } from "./context/UserDataContext";
import { RecommendationProvider } from "./context/RecommendationContext";
import "./index.css";
import "./sidebar.css";

ReactDOM.createRoot(document.getElementById("root")).render(
    <React.StrictMode>
        <AuthProvider>
            <UserDataProvider>
                <RecommendationProvider>
                    <App />
                </RecommendationProvider>
            </UserDataProvider>
        </AuthProvider>
    </React.StrictMode>,
);
