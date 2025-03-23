from django.contrib import admin
from django.urls import path, include
from django.conf import settings
from django.conf.urls.static import static
from django.contrib.auth import views as auth_views
from . import views
from django.shortcuts import render

app_name = 'InviLinkApp'

urlpatterns = [
    path('admin/', admin.site.urls),
    path('test_blockchain/', include('test_blockchain.urls')),
    path('', views.home, name='home'),
    path('admin_panel/', include('admin_panel.urls')),
    path('explore/', include('event_zone.urls')),
    path('organizer/', include('organizer.urls')),
]

def custom_404(request, exception):
    return render(request, '404.html')

handler404 = 'InviLinkApp.urls.custom_404'
