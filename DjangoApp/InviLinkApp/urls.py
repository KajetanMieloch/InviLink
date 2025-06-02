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
    path('', views.home, name='home'),
    path('admin_panel/', include('admin_panel.urls')),
    path('explore/', include('event_zone.urls')),
    path('organizer/', include('organizer.urls')),
    path('test400/', views.generate_400, name='test400'),
    path('test500/', views.generate_500, name='test500'),
]

def custom_404(request, exception):
    return render(request, '404.html')

def custom_400(request, exception):
    return render(request, '400.html', status=400)

def custom_500(request):
    return render(request, '500.html', status=500)


handler400 = 'InviLinkApp.urls.custom_400'
handler404 = 'InviLinkApp.urls.custom_404'
handler500 = 'InviLinkApp.urls.custom_500'
