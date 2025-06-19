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
    create_google_tokens_table()

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

def create_google_tokens_table():
    """Create table for storing Google tokens"""
    conn = get_db_connection()
    cursor = conn.cursor()
    cursor.execute("""
        CREATE TABLE IF NOT EXISTS user_google_tokens (
            user_id INT PRIMARY KEY,
            token_data JSON NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )
    """)
    conn.commit()
    cursor.close()
    conn.close()

# Pydantic models
class UserCreate(BaseModel):
    email: EmailStr
    username: str
    password: str

class UserLogin(BaseModel):
    email: EmailStr
    password: str

class ChatRequest(BaseModel):
    message: str
    session_id: Optional[str] = None

class Token(BaseModel):
    access_token: str
    token_type: str
    user_id: int
    username: str

class GoogleAuthRequest(BaseModel):
    auth_code: str

class GoogleAuthResponse(BaseModel):
    success: bool
    message: str

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

@app.get("/auth/google-calendar-url")
async def get_google_calendar_auth_url(current_user: dict = Depends(get_current_user)):
    """Generate Google Calendar OAuth URL for user authentication"""
    try:
        print(f"DEBUG: Starting Google OAuth URL generation for user {current_user['id']}")
        
        # Import here to avoid startup issues if google libs aren't available
        from google_auth_oauthlib.flow import Flow
        
        print("DEBUG: Google OAuth libraries imported successfully")
        
        # Get the absolute path to secrets.json (should be in project root)
        import os
        current_dir = os.path.dirname(os.path.abspath(__file__))
        project_root = os.path.dirname(current_dir)  # Go up one level from python/ to project root
        secrets_path = os.path.join(project_root, 'secrets.json')
        
        print(f"DEBUG: Looking for secrets.json at: {secrets_path}")
        print(f"DEBUG: secrets.json exists: {os.path.exists(secrets_path)}")
        
        # Configure OAuth flow
        flow = Flow.from_client_secrets_file(
            secrets_path,
            scopes=['https://www.googleapis.com/auth/calendar'],
            redirect_uri='http://localhost:3000/auth/google-callback'  # Frontend callback
        )
        
        print("DEBUG: OAuth flow configured successfully")
        
        auth_url, _ = flow.authorization_url(
            access_type='offline',
            include_granted_scopes='true',
            state=str(current_user["id"])  # Pass user ID in state
        )
        
        print(f"DEBUG: Generated auth URL successfully: {auth_url[:50]}...")
        return {"auth_url": auth_url}
        
    except Exception as e:
        print(f"ERROR: Failed to generate Google auth URL")
        print(f"ERROR: Exception type: {type(e).__name__}")
        print(f"ERROR: Exception message: {str(e)}")
        print(f"ERROR: Exception details: {repr(e)}")
        import traceback
        print(f"ERROR: Full traceback:\n{traceback.format_exc()}")
        raise HTTPException(
            status_code=500, 
            detail=f"Failed to generate Google authentication URL: {str(e)}"
        )

@app.post("/auth/google-calendar-callback", response_model=GoogleAuthResponse)
async def handle_google_calendar_callback(
    request: GoogleAuthRequest,
    current_user: dict = Depends(get_current_user)
):
    """Handle Google Calendar OAuth callback"""
    try:
        from google_auth_oauthlib.flow import Flow
        from google.auth.transport.requests import Request
        
        # Get the absolute path to secrets.json (should be in project root)
        import os
        current_dir = os.path.dirname(os.path.abspath(__file__))
        project_root = os.path.dirname(current_dir)  # Go up one level from python/ to project root
        secrets_path = os.path.join(project_root, 'secrets.json')
        
        # Configure OAuth flow
        flow = Flow.from_client_secrets_file(
            secrets_path,
            scopes=['https://www.googleapis.com/auth/calendar'],
            redirect_uri='http://localhost:3000/auth/google-callback'
        )
        
        # Exchange authorization code for tokens
        flow.fetch_token(code=request.auth_code)
        
        # Get credentials and prepare token data for storage
        credentials = flow.credentials
        token_data = {
            'token': credentials.token,
            'refresh_token': credentials.refresh_token,
            'token_uri': credentials.token_uri,
            'client_id': credentials.client_id,
            'client_secret': credentials.client_secret,
            'scopes': credentials.scopes
        }
        
        # Store token for user
        store_google_token(current_user["id"], token_data)
        
        return GoogleAuthResponse(
            success=True,
            message="Google Calendar authentication successful! Calendar events will now be created automatically."
        )
        
    except Exception as e:
        print(f"Error handling Google callback: {e}")
        return GoogleAuthResponse(
            success=False,
            message="Failed to authenticate with Google Calendar. Calendar events will be sent as messages instead."
        )

@app.get("/auth/google-calendar-status")
async def get_google_calendar_status(current_user: dict = Depends(get_current_user)):
    """Check if user has Google Calendar authentication"""
    token_data = get_google_token(current_user["id"])
    return {
        "is_authenticated": token_data is not None,
        "message": "Google Calendar is connected" if token_data else "Google Calendar not connected"
    }

# Chat endpoints (protected)
@app.post("/chat")
async def chat_endpoint(
    chat_request: ChatRequest,
    current_user: dict = Depends(get_current_user)
):
    try:
        session_id = chat_request.session_id
        user_id = current_user["id"]
        
        # Always prepend system message to conversation history
        conversation_history = get_conversation_history(user_id, session_id)
        if not conversation_history or conversation_history[0].get("role") != "system":
            conversation_history.insert(0, {"role": "system", "content": conversational_orchestrator_prompt})
        
        # Process the message with user context for calendar integration
        response, updated_history = process_message(
            chat_request.message, 
            conversation_history, 
            session_id,
            user_context={
                "user_id": user_id,
                "has_google_calendar": get_google_token(user_id) is not None,
                "google_token": get_google_token(user_id)
            }
        )
        
        # Save messages to database
        save_conversation_message(user_id, session_id, "user", chat_request.message)
        save_conversation_message(user_id, session_id, "assistant", response)
        
        return {"response": response, "session_id": session_id}
    except Exception as e:
        print(f"Chat endpoint error: {e}")
        raise HTTPException(status_code=500, detail="Internal server error")

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
    
    print(f"DEBUG: Found {len(messages)} database records for session {session_id}")
    
    # Filter out incomplete records and convert datetime objects to strings for JSON serialization
    filtered_messages = []
    for message in messages:
        # Only include complete message pairs
        if message["message"] and message["response"]:
            if 'created_at' in message and message['created_at']:
                message['created_at'] = message['created_at'].isoformat()
            filtered_messages.append(message)
        else:
            print(f"DEBUG: Skipping incomplete record - message: '{message['message']}', response: '{message['response']}'")
    
    print(f"DEBUG: Returning {len(filtered_messages)} complete messages")
    return {"messages": filtered_messages}

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
    """Get conversation history for a user and session in OpenAI message format"""
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
    
    # Convert to OpenAI message format
    formatted_messages = []
    for msg in messages:
        # Add user message
        formatted_messages.append({
            "role": "user",
            "content": msg["message"]
        })
        # Add assistant response
        formatted_messages.append({
            "role": "assistant", 
            "content": msg["response"]
        })
    
    return formatted_messages

def save_conversation_message(user_id: int, session_id: str, role: str, content: str):
    """Save a conversation message to database"""
    conn = get_db_connection()
    cursor = conn.cursor()
    
    if role == "user":
        # Insert user message - we'll update with response later
        cursor.execute("""
            INSERT INTO conversation_messages (user_id, session_id, message, response)
            VALUES (%s, %s, %s, %s)
        """, (user_id, session_id, content, ""))
    else:
        # Update the latest message with the assistant response
        cursor.execute("""
            UPDATE conversation_messages 
            SET response = %s 
            WHERE user_id = %s AND session_id = %s AND response = ""
            ORDER BY created_at DESC 
            LIMIT 1
        """, (content, user_id, session_id))
    
    conn.commit()
    cursor.close()
    conn.close()
    print(f"DEBUG: Saved {role} message to database for user {user_id}, session {session_id}")

# Add Google Calendar token storage functions
def store_google_token(user_id: int, token_data: dict):
    """Store Google Calendar token for user"""
    conn = get_db_connection()
    cursor = conn.cursor()
    token_json = json.dumps(token_data)
    cursor.execute(
        "INSERT INTO user_google_tokens (user_id, token_data) VALUES (%s, %s) "
        "ON DUPLICATE KEY UPDATE token_data = %s",
        (user_id, token_json, token_json)
    )
    conn.commit()
    cursor.close()
    conn.close()

def get_google_token(user_id: int) -> dict:
    """Retrieve Google Calendar token for user"""
    conn = get_db_connection()
    cursor = conn.cursor(dictionary=True)
    cursor.execute("SELECT token_data FROM user_google_tokens WHERE user_id = %s", (user_id,))
    result = cursor.fetchone()
    cursor.close()
    conn.close()
    
    if result:
        return json.loads(result["token_data"])
    return None
