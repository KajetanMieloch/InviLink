from django.contrib import admin
from django.urls import path, include
from django.conf import settings
from django.conf.urls.static import static
from django.contrib.auth import views as auth_views
from . import views

urlpatterns = [
    # Ścieżka do panelu administracyjnego Django
    path('admin/', admin.site.urls),

    # Ścieżka do wydarzeń (appka events)
    path('events/', include('events.urls')),

    # Główna strona aplikacji przekierowuje na listę wydarzeń
    path('', include('events.urls')), 
    path('userProfile/', include('userProfile.urls')),  # Ścieżki do aplikacji userProfile

    # Ścieżki logowania i wylogowania
    path('login/', auth_views.LoginView.as_view(template_name='registration/login.html'), name='login'),
    path('logout/', auth_views.LogoutView.as_view(), name='logout'),
    
    # Ścieżka logowania przez MetaMask
    path('phantom_login/', views.phantom_login, name='phantom_login'),
] + static(settings.STATIC_URL, document_root=settings.STATIC_ROOT)
