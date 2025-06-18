from fastapi import FastAPI, HTTPException, Depends, Cookie, Response, status
from fastapi.middleware.cors import CORSMiddleware
from fastapi.security import HTTPBearer, HTTPAuthorizationCredentials
from pydantic import BaseModel, EmailStr
from passlib.context import CryptContext
from jose import JWTError, jwt
from datetime import datetime, timedelta, timezone
from typing import Optional
import uuid
import redis
import json
import mysql.connector
import os
from dotenv import load_dotenv
from chat import create_new_conversation, process_message
from urllib.parse import urlparse
from prompts import conversational_orchestrator_prompt

load_dotenv()

app = FastAPI(title="FoodAgent API", description="AI-powered food management system")

# Add CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Initialize database tables on startup
@app.on_event("startup")
async def startup_event():
    create_messages_table()

redis_client = redis.Redis(host='localhost', port=6379, decode_responses=True)
security = HTTPBearer()
pwd_context = CryptContext(schemes=["bcrypt"], deprecated="auto")

# JWT Configuration
SECRET_KEY = os.getenv("SECRET_KEY")
ALGORITHM = "HS256"
ACCESS_TOKEN_EXPIRE_MINUTES = 30

# Database connection
def get_db_connection():
    connection_string = os.getenv("DATABASE_URL", "mysql://root:@localhost:3306/food_registry")
    
    # Parse connection string: mysql://username:password@host:port/database
    parsed = urlparse(connection_string)
    
    return mysql.connector.connect(
        host=parsed.hostname,
        port=parsed.port or 3306,
        user=parsed.username,
        password=parsed.password or "",
        database=parsed.path.lstrip('/')
    )

def create_messages_table():
    """Create the messages table if it doesn't exist"""
    conn = get_db_connection()
    cursor = conn.cursor()
    
    cursor.execute("""
        CREATE TABLE IF NOT EXISTS conversation_messages (
            id INT AUTO_INCREMENT PRIMARY KEY,
            user_id INT NOT NULL,
            session_id VARCHAR(255) NOT NULL,
            message TEXT NOT NULL,
            response TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users(id),
            INDEX idx_user_session (user_id, session_id),
            INDEX idx_created_at (created_at)
        )
    """)
    
    conn.commit()
    cursor.close()
    conn.close()
    print("DEBUG: Messages table created/verified")

# Pydantic models
class UserCreate(BaseModel):
    email: EmailStr
    username: str
    password: str

class UserLogin(BaseModel):
    email: EmailStr
    password: str

class ChatMessage(BaseModel):
    message: str
    session_id: Optional[str] = None

class Token(BaseModel):
    access_token: str
    token_type: str
    user_id: int
    username: str

# Password utilities
def verify_password(plain_password, hashed_password):
    return pwd_context.verify(plain_password, hashed_password)

def get_password_hash(password):
    return pwd_context.hash(password)

# JWT utilities
def create_access_token(data: dict, expires_delta: timedelta = None):
    to_encode = data.copy()
    if expires_delta:
        expire = datetime.now(timezone.utc) + expires_delta
    else:
        expire = datetime.now(timezone.utc) + timedelta(minutes=15)
    to_encode.update({"exp": expire})
    encoded_jwt = jwt.encode(to_encode, SECRET_KEY, algorithm=ALGORITHM)
    return encoded_jwt

async def get_current_user(credentials: HTTPAuthorizationCredentials = Depends(security)):
    credentials_exception = HTTPException(
        status_code=status.HTTP_401_UNAUTHORIZED,
        detail="Could not validate credentials",
        headers={"WWW-Authenticate": "Bearer"},
    )
    try:
        token = credentials.credentials
        payload = jwt.decode(token, SECRET_KEY, algorithms=[ALGORITHM])
        user_id: int = payload.get("sub")
        if user_id is None:
            raise credentials_exception
    except JWTError:
        raise credentials_exception
    
    # Get user from database
    conn = get_db_connection()
    cursor = conn.cursor(dictionary=True)
    cursor.execute("SELECT id, email, username FROM users WHERE id = %s", (user_id,))
    user = cursor.fetchone()
    cursor.close()
    conn.close()
    
    if user is None:
        raise credentials_exception
    return user

# Authentication endpoints
@app.post("/auth/register", response_model=Token)
async def register(user: UserCreate):
    try:
        print(f"DEBUG: Received registration data: {user}")
        print(f"DEBUG: Email: {user.email}, Username: {user.username}")
        print(f"DEBUG: Registration attempt for {user.email}")
        
        conn = get_db_connection()
        cursor = conn.cursor()
        
        # Check if user exists
        cursor.execute("SELECT id FROM users WHERE email = %s OR username = %s", (user.email, user.username))
        existing_user = cursor.fetchone()
        
        if existing_user:
            cursor.close()
            conn.close()
            print(f"DEBUG: User already exists: {user.email}")
            raise HTTPException(status_code=400, detail="Email or username already registered")
        
        # Create user
        hashed_password = get_password_hash(user.password)
        print(f"DEBUG: Password hashed successfully")
        cursor.execute(
            "INSERT INTO users (email, username, password_hash) VALUES (%s, %s, %s)",
            (user.email, user.username, hashed_password)
        )
        user_id = cursor.lastrowid
        conn.commit()
        cursor.close()
        conn.close()
        
        print(f"DEBUG: User created successfully with ID: {user_id}")
        
        # Create access token
        access_token_expires = timedelta(minutes=ACCESS_TOKEN_EXPIRE_MINUTES)
        access_token = create_access_token(
            data={"sub": str(user_id)}, expires_delta=access_token_expires
        )
        
        print(f"DEBUG: Access token created successfully")
        
        return {
            "access_token": access_token,
            "token_type": "bearer",
            "user_id": user_id,
            "username": user.username
        }
        
    except HTTPException:
        raise
    except Exception as e:
        print(f"DEBUG: Registration error: {str(e)}")
        raise HTTPException(status_code=500, detail=f"Registration failed: {str(e)}")

@app.post("/auth/login", response_model=Token)
async def login(user: UserLogin):
    conn = get_db_connection()
    cursor = conn.cursor(dictionary=True)
    cursor.execute("SELECT id, username, password_hash FROM users WHERE email = %s", (user.email,))
    db_user = cursor.fetchone()
    cursor.close()
    conn.close()
    
    if not db_user or not verify_password(user.password, db_user["password_hash"]):
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Incorrect email or password",
        )
    
    access_token_expires = timedelta(minutes=ACCESS_TOKEN_EXPIRE_MINUTES)
    access_token = create_access_token(
        data={"sub": str(db_user["id"])}, expires_delta=access_token_expires
    )
    
    return {
        "access_token": access_token,
        "token_type": "bearer",
        "user_id": db_user["id"],
        "username": db_user["username"]
    }

# Chat endpoints (protected)
@app.post("/chat")
async def chat(
    chat_request: ChatMessage,
    current_user: dict = Depends(get_current_user)
):
    print(f"DEBUG: Chat request received from user {current_user['id']}")
    print(f"DEBUG: Message: {chat_request.message}")
    print(f"DEBUG: Session ID: {chat_request.session_id}")
    user_id = current_user["id"]
    session_id = chat_request.session_id or str(uuid.uuid4())
    
    print(f"DEBUG: Using session_id: {session_id}")
    
    # Get conversation history from database
    conversation_history = get_conversation_history(user_id, session_id)
    print(f"DEBUG: Found {len(conversation_history)} previous messages")
    
    # Convert database history to format expected by chat.py
    formatted_history = []
    # if len(conversation_history) > 0:
        # formatted_history = [{"role": "system", "content": conversational_orchestrator_prompt}]
    for msg in conversation_history:
        formatted_history.extend([
            {"role": "user", "content": msg["message"]},
            {"role": "assistant", "content": msg["response"]}
        ])
    # else:
    #     formatted_history.append({"role": "system", "content": conversational_orchestrator_prompt})
    has_system_message = any(msg.get("role") == "system" for msg in formatted_history)
    if not has_system_message:
        formatted_history.insert(0, {"role": "system", "content": conversational_orchestrator_prompt})
        
    # Process message using existing chat logic
    ai_response, _ = process_message(chat_request.message, formatted_history, session_id)
    
    # Ensure ai_response is never None before saving to database
    if not ai_response:
        ai_response = "I'm processing your request. Please continue."
    
    # Save this conversation to database
    save_conversation_message(user_id, session_id, chat_request.message, ai_response)
    
    return {
        "response": ai_response, 
        "session_id": session_id,
        "user_id": user_id
    }

@app.get("/get_session_messages/{session_id}")
async def get_session_messages(session_id: str, current_user: dict = Depends(get_current_user)):
    """Get all messages for a specific session"""
    conn = get_db_connection()
    
    cursor = conn.cursor(dictionary=True)
    cursor.execute("""
        SELECT message, response, created_at 
        FROM conversation_messages 
        WHERE user_id = %s AND session_id = %s 
        ORDER BY created_at ASC 
    """, (current_user["id"], session_id))
    messages = cursor.fetchall()
    cursor.close()
    conn.close()
    
    return {"messages": messages}

@app.delete("/conversations/{session_id}")
async def delete_conversation(session_id: str, current_user: dict = Depends(get_current_user)):
    """Delete a specific conversation"""
    user_id = current_user["id"]
    
    # Delete from database
    conn = get_db_connection()
    cursor = conn.cursor()
    cursor.execute(
        "DELETE FROM user_conversations WHERE user_id = %s AND session_id = %s",
        (user_id, session_id)
    )
    affected_rows = cursor.rowcount
    conn.commit()
    cursor.close()
    conn.close()
    
    if affected_rows == 0:
        raise HTTPException(status_code=404, detail="Conversation not found")
    
    # Delete from Redis
    conversation_key = f"conversation:{user_id}:{session_id}"
    redis_client.delete(conversation_key)
    
    return {"message": "Conversation deleted successfully"}

# Health check
@app.get("/health")
async def health_check():
    return {"status": "healthy", "service": "FoodAgent API"}

# Public endpoint for testing (no auth required)
@app.get("/")
async def root():
    return {"message": "Welcome to FoodAgent API! Please register or login to start chatting."}

def get_conversation_history(user_id: int, session_id: str, limit: int = 10):
    """Get conversation history for a user and session"""
    conn = get_db_connection()
    cursor = conn.cursor(dictionary=True)
    
    cursor.execute("""
        SELECT message, response, created_at 
        FROM conversation_messages 
        WHERE user_id = %s AND session_id = %s 
        ORDER BY created_at ASC 
        LIMIT %s
    """, (user_id, session_id, limit))
    
    messages = cursor.fetchall()
    cursor.close()
    conn.close()
    
    return messages

def save_conversation_message(user_id: int, session_id: str, message: str, response: str):
    """Save a conversation message and response to database"""
    conn = get_db_connection()
    cursor = conn.cursor()
    
    cursor.execute("""
        INSERT INTO conversation_messages (user_id, session_id, message, response)
        VALUES (%s, %s, %s, %s)
    """, (user_id, session_id, message, response))
    
    conn.commit()
    cursor.close()
    conn.close()
    print(f"DEBUG: Saved message to database for user {user_id}, session {session_id}")
