from django.urls import path
from . import views
from django.conf import settings
from django.conf.urls.static import static

urlpatterns = [
    path('', views.event_list, name='event_list'),
]  + static(settings.STATIC_URL, document_root=settings.STATIC_ROOT)