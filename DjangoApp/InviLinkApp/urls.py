from django.contrib import admin
from django.urls import path, include
from django.conf import settings
from django.conf.urls.static import static
from django.contrib.auth import views as auth_views
from . import views

app_name = 'InviLinkApp'

urlpatterns = [
    path('admin/', admin.site.urls),
    path('test_blockchain/', include('test_blockchain.urls')),
    path('', views.home, name='home'),
    path('admin_panel/', include('admin_panel.urls')),
    path('explore/', include('event_zone.urls'))
] + static(settings.STATIC_URL, document_root=settings.STATIC_ROOT)
