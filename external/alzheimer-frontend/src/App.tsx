import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom'
import { Toaster } from 'react-hot-toast'
import Layout from './components/Layout'
import Dashboard from './pages/Dashboard'
import Conversation from './pages/Conversation'
import Assessment from './pages/Assessment'
import Analytics from './pages/Analytics'
import Profile from './pages/Profile'
import Login from './pages/Login'
import Register from './pages/Register'

function App() {
  // 这里应该检查用户是否已登录
  // 暂时假设用户已登录
  const isAuthenticated = true

  return (
    <Router>
      <Toaster position="top-right" />
      <Routes>
        <Route path="/login" element={<Login />} />
        <Route path="/register" element={<Register />} />

        <Route path="/" element={
          isAuthenticated ? (
            <Layout>
              <Dashboard />
            </Layout>
          ) : (
            <Navigate to="/login" />
          )
        } />

        <Route path="/conversation" element={
          isAuthenticated ? (
            <Layout>
              <Conversation />
            </Layout>
          ) : (
            <Navigate to="/login" />
          )
        } />

        <Route path="/assessment" element={
          isAuthenticated ? (
            <Layout>
              <Assessment />
            </Layout>
          ) : (
            <Navigate to="/login" />
          )
        } />

        <Route path="/analytics" element={
          isAuthenticated ? (
            <Layout>
              <Analytics />
            </Layout>
          ) : (
            <Navigate to="/login" />
          )
        } />

        <Route path="/profile" element={
          isAuthenticated ? (
            <Layout>
              <Profile />
            </Layout>
          ) : (
            <Navigate to="/login" />
          )
        } />
      </Routes>
    </Router>
  )
}

export default App