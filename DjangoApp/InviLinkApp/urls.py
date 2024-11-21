from django.contrib import admin
from django.urls import path, include
from django.conf import settings
from django.conf.urls.static import static
from django.contrib.auth import views as auth_views
from . import views

urlpatterns = [
    path('admin/', admin.site.urls),
    path('events/', include('events.urls')),

    path('', include('events.urls')), 
    path('userProfile/', include('userProfile.urls')),

    path('login/', auth_views.LoginView.as_view(template_name='registration/login.html'), name='login'),
    path('logout/', auth_views.LogoutView.as_view(), name='logout'),
    
    path('', include('auth_blockchain.urls')),
    path('phantom_login/', views.phantom_login, name='phantom_login'),
    path('auth/', include('auth_blockchain.urls')),
    path('test_blockchain/', include('test_blockchain.urls')),
] + static(settings.STATIC_URL, document_root=settings.STATIC_ROOT)
